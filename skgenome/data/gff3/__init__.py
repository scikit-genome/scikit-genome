import os.path

import skgenome.data


def human(release="99"):
    filename = f"human.{release}.gff3"

    origin = f"ftp://ftp.ensembl.org/pub/release-{release}/gff3/homo_sapiens/Homo_sapiens.GRCh38.{release}.gff3.gz"

    pathname = skgenome.data.get(filename, origin)

    return pathname
