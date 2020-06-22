mod io;

extern crate buf_redux;
extern crate memchr;

#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod fasta;

use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};

#[pyfunction]
fn foo(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn ext(_: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pyfunction!(foo))?;
    Ok(())
}

#[pymodule]
fn skgenome(_: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pymodule!(ext))?;

    Ok(())
}
