import os.path


def human(release="99"):
    resource = os.path.join(
        "ftp://ftp.ensembl.org",
        f"pub/release-{release}/gff3/homo_sapiens/",
        f"/Homo_sapiens.GRCh38.{release}.gff3.gz",
    )
