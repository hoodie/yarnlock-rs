#![allow(dead_code)]
extern crate indent_tokenizer;
#[macro_use]
extern crate nom;

use std::collections::HashMap;

#[derive(Debug)]
pub struct DependencyLock {
    pub name: String,
    pub last_seen: String,  // VersionReq
    pub properties: HashMap<String, String>,
    pub dependencies: HashMap<String, String>,
    // pub resolved: String,  // url
}

pub mod parser;

#[cfg(test)]
mod tests {
    use super::parser;

    fn test_file() -> &'static str {
        include_str!("../yarn.lock")
    }

    #[test]
    fn parses_head_lines() {
        let p0 = parser::headline_parts(r#""@ava/babel-plugin-throws-helper@^2.0.0""#);
        let p1 = parser::headline_parts(r#"assertion-error@^1.0.1, assertion-error@^1.0.1"#);
        let p2 = parser::headline_parts(r#""@protobufjs/aspromise@^1.1.1","@protobufjs/aspromise@^1.1.2""#);
        println!("{:#?}", (p0, p1, p2));
    }

    #[test]
    fn tuple_line() {
        let p0 = parser::tuple_line(r#"version "1.4.0""#);
        let p1 = parser::tuple_line(r#"camelcase "^1.0.2""#);
        let p2 = parser::tuple_line(r#"cliui "^2.1.0""#);
        let p3 = parser::tuple_line(r#"decamelize "^1.0.0""#);
        let p4 = parser::tuple_line(r#"window-size "0.1.0""#);

        println!("{:#?}", (p0, p1, p2, p3, p4));
    }

    #[test]
    fn print() {
        let file = test_file();
        let parsed = parser::parse(file);
        println!("{:#?}", parsed);
    }
}
