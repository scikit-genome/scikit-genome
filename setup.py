import glob
import io
import os
import setuptools

directory = os.path.abspath(os.path.dirname(__file__))

with io.open(os.path.join(directory, "README.md"), encoding="utf-8") as f:
    long_description = f.read()

setuptools.setup(
    author="Allen Goodman",
    author_email="allen.goodman@icloud.com",
    classifiers=[
        "Development Status :: 1 - Planning",
        "Intended Audience :: Science/Research",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Topic :: Scientific/Engineering :: Bio-Informatics",
    ],
    description="A Python package for genomics",
    extras_require={
        "build": ["twine>=3.1.1"],
        "dev": ["black>=19.10b0", "check-manifest>=0.41", "pre-commit>=2.2.0"],
        "test": ["coverage>=5.0.4"],
    },
    install_requires=[],
    long_description=long_description,
    long_description_content_type="text/markdown",
    name="scikit-genome",
    package_data={"skgenome": glob.glob("*")},
    packages=["skgenome"],
    project_urls={
        "Bug Reports": "https://github.com/scikit-genome/scikit-genome/issues",
        "Source": "https://github.com/scikit-genome/scikit-genome/",
    },
    python_requires=">=3.7, <4",
    url="https://github.com/scikit-genome/scikit-genome",
    version="0.0.0",
)
