import os.path

import pytest

package_directory = os.path.abspath(os.path.dirname(__file__))

resources_directory = os.path.join(package_directory, "resources")


@pytest.fixture
def example_fasta_pathname():
    return os.path.join(resources_directory, "example.fasta")


@pytest.fixture
def example_fastq_pathname():
    return os.path.join(resources_directory, "example.fastq")


@pytest.fixture
def example_gff_pathname():
    return os.path.join(resources_directory, "example.gff")


@pytest.fixture
def example_sam_pathname():
    return os.path.join(resources_directory, "example.sam")


@pytest.fixture
def example_vcf_pathname():
    return os.path.join(resources_directory, "example.vcf")
