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
fn read(path: String) {
    let mut sequences: Reader<fs::File> = Reader::from_path(&path).unwrap();

    while let Some(sequence) = sequences.next() {
        println!("{}", str::from_utf8(sequence.unwrap().description()).unwrap());
    }
}

#[pymodule]
pub fn fasta(_: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pyfunction!(read))?;

    Ok(())
}

#[pymodule]
pub fn io(_: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pymodule!(fasta))?;

    Ok(())
}

#[pymodule]
fn ext(_: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pymodule!(io))?;
    Ok(())
}

#[pymodule]
fn skgenome(_: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pymodule!(ext))?;

    Ok(())
}
