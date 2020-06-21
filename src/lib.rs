extern crate buf_redux;
extern crate memchr;

#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod fasta;

macro_rules! try_opt {
    ($expr: expr) => {
        match $expr {
            Ok(item) => item,
            Err(e) => return Some(Err(::std::convert::From::from(e))),
        }
    };
}

macro_rules! unwrap_or {
    ($expr:expr, $or:block) => {
        match $expr {
            Some(item) => item,
            None => $or,
        }
    };
}

pub mod buffer_policy;
pub mod reader;

use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};

/// Remove a final '\r' from a byte slice
#[inline]
fn trim_cr(line: &[u8]) -> &[u8] {
    if let Some((&b'\r', remaining)) = line.split_last() {
        remaining
    } else {
        line
    }
}

/// Makes sure the buffer is full after this call (unless EOF reached)
/// code adapted from `io::Read::read_exact`
fn fill_buf<R>(reader: &mut buf_redux::BufReader<R, buf_redux::policy::StdPolicy>) -> std::io::Result<usize> where R: std::io::Read {
    let initial_size = reader.buffer().len();
    let mut num_read = 0;
    while initial_size + num_read < reader.capacity() {
        match reader.read_into_buf() {
            Ok(0) => break,
            Ok(n) => num_read += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(num_read)
}

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
