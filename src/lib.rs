#![allow(dead_code)]
#[macro_use]
extern crate failure;
extern crate indent_tokenizer;
#[macro_use]
extern crate log;
extern crate multimap;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate pretty_assertions;
extern crate semver;
extern crate semver_parser;
extern crate url;

use multimap::MultiMap;
use semver::{Version, VersionReq};
use url::Url;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::result::Result;

mod parser;
pub use parser::{parse, parse_by_name};
pub mod error;

pub mod npm_semver;

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

fn read_file(path: &Path) -> Result<String, error::Error> {
    let mut file = if path.is_dir() {
        let file_path = path.join("yarn.lock");
        debug!("opening {:?}", file_path);
        File::open(file_path)?
    } else {
        debug!("opening {:?}", path);
        File::open(path)?
    };
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    Ok(content)
}

/// Opens a given file or the `yarn.lock` if a folder is given.
pub fn open<P: AsRef<OsStr> + Sized>(given_path: P) -> Result<Vec<DependencyLock>, error::Error> {
    let path = Path::new(&given_path);
    read_file(&path).and_then(|s| parse(&s))
}

pub fn open_by_name<P: AsRef<OsStr> + Sized>(given_path: P) -> Result<MultiMap<String, DependencyLock>, error::Error> {
    let path = Path::new(&given_path);
    read_file(&path).and_then(|s| parse_by_name(&s))
}

impl fmt::Display for DependencyLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let version = self.version
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(String::new);

        let last_seen = self.last_seen
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        write!(f, "{}\n\t{:?} -> {}", self.name, last_seen, version)?;
        Ok(())
    }
}
