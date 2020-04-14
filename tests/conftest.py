import os.path

import pytest

package_directory = os.path.abspath(os.path.dirname(__file__))

resources_directory = os.path.join(package_directory, "resources")


@pytest.fixture
def fasta():
    return os.path.join(resources_directory, "example.fasta")


@pytest.fixture
def fastq():
    return os.path.join(resources_directory, "example.fastq")


@pytest.fixture
def gff():
    return os.path.join(resources_directory, "example.gff")


@pytest.fixture
def sam():
    return os.path.join(resources_directory, "example.sam")


@pytest.fixture
def vcf():
    return os.path.join(resources_directory, "example.vcf")
