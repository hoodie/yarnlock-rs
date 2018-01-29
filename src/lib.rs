#![allow(dead_code)]
extern crate indent_tokenizer;
#[macro_use]
extern crate nom;
extern crate semver;
extern crate url;

use url::Url;
use semver::{Version, VersionReq};

use std::collections::HashMap;

mod parser;
pub use parser::parse;

#[derive(Debug)]
pub struct DependencyLock {
    pub name: String,
    pub last_seen: Option<VersionReq>,
    pub version: Option<Version>,
    pub resolved: Option<Url>,
    pub dependencies: HashMap<String, VersionReq>,
}

