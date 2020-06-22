extern crate buf_redux;
extern crate memchr;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::{fs, str};

use pyo3::{wrap_pyfunction, wrap_pymodule};
use pyo3::prelude::*;

use crate::io::fasta::reader::Reader;
use crate::io::fasta::sequence::Record;

pub mod io;

#[pyfunction]
fn read_fasta(path: String) {
    let mut sequences: Reader<fs::File> = Reader::from_path(&path).unwrap();

    while let Some(sequence) = sequences.next() {
        println!("{}", str::from_utf8(sequence.unwrap().description()).unwrap());
    }
}

#[pymodule]
fn skgenome(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(read_fasta))?;

    Ok(())
}
