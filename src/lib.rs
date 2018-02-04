#![allow(dead_code)]
#[macro_use]
extern crate failure;
extern crate indent_tokenizer;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate semver;
extern crate url;

use semver::{Version, VersionReq};
use url::Url;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::result::Result;

mod parser;
pub use parser::parse;
pub mod error;

/// Represents one dependency Lock.
///
/// One block in your `yarn.lock` be result in multiple `DependencyLock`s.
#[derive(Debug)]
pub struct DependencyLock {
    pub name:         String,
    pub last_seen:    Option<VersionReq>,
    pub version:      Option<Version>,
    pub resolved:     Option<Url>,
    pub dependencies: HashMap<String, VersionReq>,
}

/// Opens a given file or the `yarn.lock` if a folder is given.
pub fn open<P: AsRef<OsStr> + Sized>(given_path: P) -> Result<Vec<DependencyLock>, error::Error> {
    let path = Path::new(&given_path);
    let mut file = if path.is_dir() {
        let file_path = path.join("yarn.lock");
        debug!("opening {:?}", file_path);
        File::open(file_path)?
    } else {
        debug!("opening {:?}", path);
        File::open(path)?
    };
    let content = {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        content
    };
    parse(&content)
}
