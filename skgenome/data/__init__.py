import hashlib
import os.path
import shutil
import sys
import tarfile
import time
import urllib.error
import urllib.request
import zipfile

import numpy


class ProgressBar:
    def __init__(
        self,
        target,
        width=30,
        verbose=1,
        interval=0.05,
        stateful_metrics=None,
        unit_name="step",
    ):
        self.target = target
        self.width = width
        self.verbose = verbose
        self.interval = interval
        self.unit_name = unit_name

        if stateful_metrics:
            self.stateful_metrics = set(stateful_metrics)
        else:
            self.stateful_metrics = set()

        self._dynamic_display = (
            (hasattr(sys.stdout, "isatty") and sys.stdout.isatty())
            or "ipykernel" in sys.modules
            or "posix" in sys.modules
            or "PYCHARM_HOSTED" in os.environ
        )
        self._total_width = 0
        self._seen_so_far = 0
        # We use a dict + list to avoid garbage collection
        # issues found in OrderedDict
        self._values = {}
        self._values_order = []
        self._start = time.time()
        self._last_update = 0

    def update(self, current, values=None, finalize=None):
        """Updates the progress bar.
    Arguments:
        current: Index of current step.
        values: List of tuples: `(name, value_for_last_step)`. If `name` is in
          `stateful_metrics`, `value_for_last_step` will be displayed as-is.
          Else, an average of the metric over time will be displayed.
        finalize: Whether this is the last update for the progress bar. If
          `None`, defaults to `current >= self.target`.
    """
        if finalize is None:
            if self.target is None:
                finalize = False
            else:
                finalize = current >= self.target

        values = values or []
        for k, v in values:
            if k not in self._values_order:
                self._values_order.append(k)
            if k not in self.stateful_metrics:
                # In the case that progress bar doesn't have a target value in the first
                # epoch, both on_batch_end and on_epoch_end will be called, which will
                # cause 'current' and 'self._seen_so_far' to have the same value. Force
                # the minimal value to 1 here, otherwise stateful_metric will be 0s.
                value_base = max(current - self._seen_so_far, 1)
                if k not in self._values:
                    self._values[k] = [v * value_base, value_base]
                else:
                    self._values[k][0] += v * value_base
                    self._values[k][1] += value_base
            else:
                # Stateful metrics output a numeric value. This representation
                # means "take an average from a single value" but keeps the
                # numeric formatting.
                self._values[k] = [v, 1]
        self._seen_so_far = current

        now = time.time()
        info = " - %.0fs" % (now - self._start)
        if self.verbose == 1:
            if now - self._last_update < self.interval and not finalize:
                return

            prev_total_width = self._total_width
            if self._dynamic_display:
                sys.stdout.write("\b" * prev_total_width)
                sys.stdout.write("\r")
            else:
                sys.stdout.write("\n")

            if self.target is not None:
                numdigits = int(numpy.log10(self.target)) + 1
                bar = ("%" + str(numdigits) + "d/%d [") % (current, self.target)
                prog = float(current) / self.target
                prog_width = int(self.width * prog)
                if prog_width > 0:
                    bar += "=" * (prog_width - 1)
                    if current < self.target:
                        bar += ">"
                    else:
                        bar += "="
                bar += "." * (self.width - prog_width)
                bar += "]"
            else:
                bar = "%7d/Unknown" % current

            self._total_width = len(bar)
            sys.stdout.write(bar)

            if current:
                time_per_unit = (now - self._start) / current
            else:
                time_per_unit = 0

            if self.target is None or finalize:
                if time_per_unit >= 1 or time_per_unit == 0:
                    info += " %.0fs/%s" % (time_per_unit, self.unit_name)
                elif time_per_unit >= 1e-3:
                    info += " %.0fms/%s" % (time_per_unit * 1e3, self.unit_name)
                else:
                    info += " %.0fus/%s" % (time_per_unit * 1e6, self.unit_name)
            else:
                eta = time_per_unit * (self.target - current)
                if eta > 3600:
                    eta_format = "%d:%02d:%02d" % (
                        eta // 3600,
                        (eta % 3600) // 60,
                        eta % 60,
                    )
                elif eta > 60:
                    eta_format = "%d:%02d" % (eta // 60, eta % 60)
                else:
                    eta_format = "%ds" % eta

                info = " - ETA: %s" % eta_format

            for k in self._values_order:
                info += " - %s:" % k
                if isinstance(self._values[k], list):
                    avg = numpy.mean(self._values[k][0] / max(1, self._values[k][1]))
                    if abs(avg) > 1e-3:
                        info += " %.4f" % avg
                    else:
                        info += " %.4e" % avg
                else:
                    info += " %s" % self._values[k]

            self._total_width += len(info)
            if prev_total_width > self._total_width:
                info += " " * (prev_total_width - self._total_width)

            if finalize:
                info += "\n"

            sys.stdout.write(info)
            sys.stdout.flush()

        elif self.verbose == 2:
            if finalize:
                numdigits = int(numpy.log10(self.target)) + 1
                count = ("%" + str(numdigits) + "d/%d") % (current, self.target)
                info = count + info
                for k in self._values_order:
                    info += " - %s:" % k
                    avg = numpy.mean(self._values[k][0] / max(1, self._values[k][1]))
                    if avg > 1e-3:
                        info += " %.4f" % avg
                    else:
                        info += " %.4e" % avg
                info += "\n"

                sys.stdout.write(info)
                sys.stdout.flush()

        self._last_update = now

    def add(self, n, values=None):
        self.update(self._seen_so_far + n, values)


class ProgressMonitor:
    progress_bar = None


def generate(pathname, method="sha256", chunk_size=65535):
    if (method == "sha256") or (method == "auto" and len(hash) == 64):
        f = hashlib.sha256()
    else:
        f = hashlib.md5()

    with open(pathname, "rb") as fp:
        for chunk in iter(lambda: fp.read(chunk_size), b""):
            f.update(chunk)

    return f.hexdigest()


def verify(pathname, checksum, method="auto", chunk_size=65535):
    if (method == "sha256") or (method == "auto" and len(checksum) == 64):
        f = "sha256"
    else:
        f = "md5"

    if str(generate(pathname, f, chunk_size)) == str(checksum):
        return True
    else:
        return False


def extract_archive(source, destination=".", archive_format="auto"):
    if archive_format is None:
        return False

    if archive_format == "auto":
        archive_format = ["tar", "zip"]

    archive_format = [archive_format]

    if isinstance(source, os.PathLike):
        source = os.fspath(source)

    if isinstance(destination, os.PathLike):
        destination = os.fspath(destination)

    for archive_type in archive_format:
        _is, _open = None, None

        if archive_type == "tar":
            _is, _open = tarfile.is_tarfile, tarfile.open

        if archive_type == "zip":
            _is, _open = zipfile.is_zipfile, zipfile.ZipFile

        if _is(source):
            with _open(source) as archive:
                try:
                    archive.extractall(destination)
                except (tarfile.TarError, RuntimeError, KeyboardInterrupt):
                    if os.path.exists(destination):
                        if os.path.isfile(destination):
                            os.remove(destination)
                        else:
                            shutil.rmtree(destination)
                    raise

            return True

    return False


def get(
    filename,
    origin,
    archive_format="auto",
    cache_directory=None,
    cache_subdirectory="data",
    checksum=None,
    checksum_algorithm="auto",
    extract=False,
):
    if cache_directory is None:
        cache_directory = os.path.join(os.path.expanduser("~"), ".skgenome")

    skgenome_directory = os.path.expanduser(cache_directory)

    if not os.access(skgenome_directory, os.W_OK):
        skgenome_directory = os.path.join("/tmp", ".skgenome")

    data_directory = os.path.join(skgenome_directory, cache_subdirectory)

    os.makedirs(data_directory, exist_ok=True)

    if isinstance(filename, os.PathLike):
        filename = os.fspath(filename)

    pathname = os.path.join(data_directory, filename)

    download = False

    if os.path.exists(pathname):
        if checksum is not None:
            if not verify(pathname, checksum, checksum_algorithm):
                download = True
    else:
        download = True

    if download:

        def progress(count, block_size, total_size):
            if ProgressMonitor.progress_bar is None:
                if total_size == -1:
                    total_size = None

                ProgressMonitor.progress_bar = ProgressBar(total_size)
            else:
                ProgressMonitor.progress_bar.update(count * block_size)

        try:
            try:
                urllib.request.urlretrieve(origin, pathname, progress)
            except urllib.error.HTTPError:
                raise Exception
            except urllib.error.URLError:
                raise Exception
        except (Exception, KeyboardInterrupt):
            if os.path.exists(pathname):
                os.remove(pathname)
            raise

        ProgressMonitor.progress_bar = None

    if extract:
        extract_archive(pathname, data_directory, archive_format)

    return pathname
