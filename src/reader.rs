use std::borrow::Cow;
use std::fs::File;
use std::io::{self, BufRead, Seek};
use std::iter;
use std::path::Path;
use std::slice;
use std::str::{self, Utf8Error};

use buf_redux;
use memchr::Memchr;

use super::buffer_policy::{BufferPolicy, StandardPolicy};
use super::*;

type DefaultPolicy = StandardPolicy;

const BUFFER_SIZE: usize = 64 * 1024;



