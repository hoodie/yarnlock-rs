#![allow(unused_parens)]

use indent_tokenizer::{tokenize, Token};
use nom::{line_ending, IResult};
use semver::{Version, VersionReq};
use url::Url;
use multimap::MultiMap;

use std::collections::HashMap;
use std::str::from_utf8;
use std::ops::Deref;

use super::DependencyLock;
use error;

fn read_versionreq(tokens: &[Token]) -> Option<VersionReq> {
    tokens
        .iter()
        .flat_map(|t| &t.lines)
        .filter_map(|line| {
            let tup_line = versionreq_line(line);
            if let IResult::Done(_left_overs, tup_line) = tup_line {
                Some(tup_line)
            } else {
                error!("INVALID VersionRequirement {:?}", line);
                None
            }
        })
        .nth(0)
}

fn read_version_resolved(tokens: &[Token]) -> (Option<Version>, Option<Url>) {
    let mut version = None;
    let mut resolved = None;
    for line in tokens.iter().flat_map(|t| &t.lines) {
        let tup_line = version_line(line);
        if let IResult::Done(_left_overs, ver) = tup_line {
            version = Some(ver);
        }
        let tup_line = resolved_line(line);
        if let IResult::Done(_left_overs, res) = tup_line {
            resolved = Some(res);
        }
    }
    (version, resolved)
}

fn read_dependencies(tokens: &[Token]) -> HashMap<String, VersionReq> {
    tokens
        .iter()
        .filter(|token| {
            token
                .lines
                .last()
                .map(|s| s.starts_with("dependencies"))
                .unwrap_or(false)
        })
        .flat_map(|t| &t.tokens)
        .nth(0)
        .iter()
        .flat_map(|t| &t.lines)
        .filter_map(|line| {
            let tup_line = dependency_line(line);
            if let IResult::Done(_left_overs, tup_line) = tup_line {
                let (key, val) = tup_line;
                Some((key.to_string(), val))
            } else {
                error!("INVALID Depedency {}", line.deref());
                None
            }
        })
        .collect()
}

fn read_block(block: &Token) -> Vec<DependencyLock> {
    let dependencies = read_dependencies(&block.tokens);
    let (version, resolved) = read_version_resolved(&block.tokens);

    block
        .lines
        .iter()
        .filter(|l| !l.starts_with('#'))
        .flat_map(|heading| {
            let (_, head_lines) = headline_parts(&heading).unwrap();
            head_lines
                .into_iter()
                .map(|(last_seen, name)| DependencyLock {
                    name: name.unwrap().to_string(),
                    last_seen: last_seen.and_then(|s| VersionReq::parse(s).ok()),
                    version: version.clone(),
                    resolved: resolved.clone(),
                    dependencies: dependencies.clone(),
                })
        })
        .collect()
}

fn split_at_last_at(s: &str) -> (Option<&str>, Option<&str>) {
    let mut it = s.rsplitn(2, '@');
    (it.nth(0), it.nth(0))
}

/// Parses content of a `yarn.lock` into a `Vec<DepdencencyLock>`.
pub fn parse(content: &str) -> Result<Vec<DependencyLock>, error::Error> {
    Ok(tokenize(content)
        .map_err(error::IndentationFail)?
        .iter()
        .flat_map(read_block)
        .collect())
}

/// Parses content of a `yarn.lock` and maps the linto a `MultiMap<Strign, DepdencencyLock>`.
pub fn parse_by_name(content: &str) -> Result<MultiMap<String, DependencyLock>, error::Error> {
    Ok(tokenize(content)
        .map_err(error::IndentationFail)?
        .iter()
        .flat_map(read_block)
        .map(|lock| (lock.name.clone(), lock))
        .collect())
}

fn headline_parts(content: &str) -> IResult<&[u8], Vec<(Option<&str>, Option<&str>)>> {
    at_tuple_list(content.as_bytes())
}

named!{
headline(&[u8]) -> &str,
    do_parse!(
        content: map_res!(take_till_s!(|c| c == ':' as u8), from_utf8) >>
        opt!(line_ending) >>
        (content)
    )
}

named!{
at_tuple(&[u8]) -> (Option<&str>, Option<&str>),
    map!(
        map_res!(
            alt!( is_not!(",\"")
                | delimited!(char!('"'), is_not!(":\""), char!('"'))
                )
        , from_utf8)
    , split_at_last_at)
}

named!{
at_tuple_list(&[u8]) -> Vec<(Option<&str>, Option<&str>)>,
    separated_list_complete!(ws!(tag!(",")), at_tuple)
}

// named!{
// headline_parts2_int(&[u8]) -> (Option<&str>, Option<VersionReq>),
//          alt!(
//              map!( map_res!(is_not!(":,\""), from_utf8), split_at_last_at)
//              |
//              map!( map_res!(delimited!(char!('"'), is_not!(":,\""), char!('"')), from_utf8), split_at_last_at)
//              )
// }

fn dependency_line(content: &str) -> IResult<&[u8], (&str, VersionReq)> {
    dependency_line_int(content.as_bytes())
}

named!{
dependency_line_int(&[u8]) -> (&str, VersionReq),
    ws!(tuple!(alt!( map_res!(delimited!(char!('"'), is_not!(",\""), char!('"')), from_utf8)
                   | map_res!(is_not!(" "), from_utf8)
                   ),
        map_res!(
        map_res!(delimited!(char!('"'), is_not!(",\""), char!('"')), from_utf8)
        , VersionReq::parse)
    ))
}

fn version_line(content: &str) -> IResult<&[u8], Version> {
    version_line_int(content.as_bytes())
}

named!{
version_line_int(&[u8]) -> Version,
    map_res!(
    ws!(do_parse!(
        tag!("version") >>
        version: alt!( map_res!(delimited!(char!('"'), is_not!(",\""), char!('"')), from_utf8)
                     | map_res!(is_not!(" "), from_utf8)
                     )
        >> (version)
        )
    ), Version::parse)
}

fn versionreq_line(content: &str) -> IResult<&[u8], VersionReq> {
    versionreq_line_int(content.as_bytes())
}

named!{
versionreq_line_int(&[u8]) -> VersionReq,
    map_res!(
    ws!(do_parse!(
        tag!("version") >>
        versionreq: alt!( map_res!(delimited!(char!('"'), is_not!(",\""), char!('"')), from_utf8)
                        | map_res!(is_not!(" "), from_utf8)
                        )
        >> (versionreq)
        )
    ), VersionReq::parse)
}

fn resolved_line(content: &str) -> IResult<&[u8], Url> {
    resolved_line_int(content.as_bytes())
}

named!{
resolved_line_int(&[u8]) -> Url,
        map_res!(
        ws!(do_parse!(
            tag!("resolved") >>
            resolved: alt!( map_res!(delimited!(char!('"'), is_not!(",\""), char!('"')), from_utf8)
                          | map_res!(is_not!(" "), from_utf8)
                          )
            >> (resolved)
            )
        ), Url::parse)
}

#[cfg(test)]
mod tests {
    #![allow(unused_macros)]

    use super::*;

    fn test_file() -> &'static str {
        include_str!("../yarn.lock.big")
    }

    macro_rules! assert_parser {
        ($parser: expr, $expected: expr) => {
            assert_eq!($parser, IResult::Done(&[][..], $expected));
        };
    }

    #[test]
    fn parses_version_lines() {
        assert_parser!(
            version_line(r#"version 3.0.0"#),
            Version::parse("3.0.0").unwrap()
        );
        assert_parser!(
            version_line(r#"version "3.0.0""#),
            Version::parse("3.0.0").unwrap()
        );
    }

    #[test]
    fn parses_head_lines() {
        assert_parser!(
            headline_parts(r#""@protobufjs/aspromise@^1.1.1","@protobufjs/aspromise@^1.1.2""#),
            vec![
                (Some("^1.1.1"), Some("@protobufjs/aspromise")),
                (Some("^1.1.2"), Some("@protobufjs/aspromise")),
            ]
        );

        assert_parser!(
            headline_parts(r#""@ava/babel-plugin-throws-helper@^2.0.0""#),
            vec![(Some("^2.0.0"), Some("@ava/babel-plugin-throws-helper"))]
        );

        assert_parser!(
            headline_parts(r#""@ava/babe,-plugin-throws-helper@^2.0.0", "@ava/babel-plugin-throws-helper@^2.0.0""#),
            vec![(Some("^2.0.0"), Some("@ava/babe,-plugin-throws-helper")),
                 (Some("^2.0.0"), Some("@ava/babel-plugin-throws-helper"))]
        );

        assert_parser!(
            headline_parts(r#"assertion-error@^1.0.1, assertion-error@^1.0.1"#),
            vec![
                (Some("^1.0.1"), Some("assertion-error")),
                (Some("^1.0.1"), Some("assertion-error")),
            ]
        );
    }

    #[test]
    fn parses_head_lines_deep() {
        assert_parser!(
            headline_parts("fstream@>= 0.1.30 < 1"),
            vec![
                (
                    Some(">= 0.1.30 < 1"),
                    Some("fstream"),
                )

            ]);
    }

    #[test]
    fn parses_dependency_lines() {
        assert_parser!(
            dependency_line(r#"version "1.4.0""#),
            ("version", VersionReq::parse("1.4.0").unwrap())
        );
        assert_parser!(
            dependency_line(r#"camelcase "^1.0.2""#),
            ("camelcase", VersionReq::parse("^1.0.2").unwrap())
        );
        assert_parser!(
            dependency_line(r#"cliui "^2.1.0""#),
            ("cliui", VersionReq::parse("^2.1.0").unwrap())
        );
        assert_parser!(
            dependency_line(r#"decamelize "^1.0.0""#),
            ("decamelize", VersionReq::parse("^1.0.0").unwrap())
        );
        assert_parser!(
            dependency_line(r#"window-size "0.1.0""#),
            ("window-size", VersionReq::parse("0.1.0").unwrap())
        );
        assert_parser!(
            dependency_line(r#""window-size" "0.1.0""#),
            ("window-size", VersionReq::parse("0.1.0").unwrap())
        );
    }

    #[test]
    #[ignore]
    fn read_dependencies() {
        let samples = [
            r#"through ">=2.2.7 <3""#,
            r#"readable-stream "^2.0.0 || ^1.1.13""#,
            r#"readable-stream "> 1.0.0 < 3.0.0""#,
            r#"traverse ">=0.3.0 <0.4""#,
            r#"readable-stream "1 || 2""#,
            r#"mkdirp ">=0.5 0""#,
            r#"minimatch "2 || 3""#,
            r#"statuses ">= 1.3.1 < 2""#,
            r#"npm-package-arg "^4.0.0 || ^5.0.0""#,
            r#"read-package-json "1 || 2""#,
            r#"semver "2.x || 3.x || 4 || 5""#,
            r#"nopt "2 || 3""#,
            r#"npmlog "0 || 1 || 2 || 3 || 4""#,
            r#"semver "2 || 3 || 4 || 5""#,
            r#"semver "^2.3.0 || 3.x || 4 || 5""#,
            r#"normalize-package-data "~1.0.1 || ^2.0.0""#,
            r#"npm-package-arg "^3.0.0 || ^4.0.0 || ^5.0.0""#,
            r#"semver "2 >=2.2.1 || 3.x || 4 || 5""#,
            r#"semver "2 || 3 || 4""#,
            r#"over ">= 0.0.5 < 1""#,
            r#"setimmediate ">= 1.0.2 < 2""#,
            r#"slice-stream ">= 1.0.0 < 2""#,
            r#"semver "2 || 3 || 4 || 5""#,
            r#"util ">=0.10.3 <1""#,
            r#"thenify ">= 3.1.0 < 4""#,
            r#"binary ">= 0.3.0 < 1""#,
            r#"fstream ">= 0.1.30 < 1""#,
            r#"match-stream ">= 0.0.2 < 1""#,
            r#"pullstream ">= 0.4.1 < 1""#,
            r#"setimmediate ">= 1.0.1 < 2""#,
        ];
        let sample = samples[0];
        assert_parser!(
            dependency_line(sample),
            ("through", VersionReq::parse("2").unwrap())
        );
    }

    #[test]
    fn print() {
        let file = test_file();
        let parsed = parse(file);
        println!("{:#?}", parsed);
    }
}
