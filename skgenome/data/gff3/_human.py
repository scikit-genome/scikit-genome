import pandas

import skgenome.data


def human(release="99"):
    filename = f"human.{release}.gff3.gz"

    origin = f"ftp://ftp.ensembl.org/pub/release-{release}/gff3/homo_sapiens/Homo_sapiens.GRCh38.{release}.gff3.gz"

    pathname = skgenome.data.get(filename, origin)

    names = [
        "seqid",
        "source",
        "type",
        "start",
        "end",
        "score",
        "strand",
        "phase",
        "attributes",
    ]

    return pandas.read_csv(
        pathname,
        comment="#",
        compression="gzip",
        low_memory=False,
        names=names,
        sep="\t",
    )
