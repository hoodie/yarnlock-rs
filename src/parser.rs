#![allow(unused_parens)]

use indent_tokenizer::{self, tokenize, Token};
use nom::{line_ending, IResult};
use url::Url;
use semver::{Version, VersionReq};

use std::collections::HashMap;
use std::str::from_utf8;

use super::DependencyLock;

fn read_versionreq(tokens: &[Token]) -> Option<VersionReq> {
    tokens
        .iter()
        .flat_map(|t| &t.lines)
        .filter_map(|line| {
            let tup_line = versionreq_line(line);
            if let IResult::Done(_left_overs, tup_line) = tup_line {
                Some(tup_line)
            } else {
                None
            }
        })
        .nth(0)
}

fn read_version_resolved(tokens: &[Token]) -> (Option<Version>, Option<Url>) {
    let mut version = None;
    let mut resolved = None;
    for line in tokens .iter() .flat_map(|t| &t.lines) {
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
    tokens.iter()
        .filter(|token| token.lines.last().map(|s| s.starts_with("dependencies")).unwrap_or(false))
        .flat_map(|t| &t.tokens)
        .nth(0).iter()
        .flat_map(|t| &t.lines)
        //.map(|sline| String::from(sline.as_ref()))
        .map(|sline| sline)
            .filter_map(|level2| {
                let tup_line = dependency_line(level2);
                if let IResult::Done(_left_overs, tup_line) = tup_line {
                    let (key, val) = tup_line;
                    Some((key.to_string(), val))
                } else {
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
                    name:         name.unwrap().to_string(),
                    last_seen:    last_seen.and_then(|s| VersionReq::parse(s).ok()),
                    version:      version.clone(),
                    resolved:     resolved.clone(),
                    // properties:   properties.clone(),
                    dependencies: dependencies.clone(),
                })
        })
        .collect()
}

fn split_at_last_at(s: &str) -> (Option<&str>, Option<&str>) {
    let mut it = s.rsplitn(2, '@');
    (it.nth(0), it.nth(0))
}



// TODO: Use own errors
pub fn parse(content: &str) -> Result<Vec<DependencyLock>, indent_tokenizer::Error> {
    Ok(tokenize(content)?.iter().flat_map(read_block).collect())
}

fn headline_parts(content: &str) -> IResult<&[u8], Vec<(Option<&str>, Option<&str>)>> {
    headline_parts_int(content.as_bytes())
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
headline_parts_int(&[u8]) -> Vec<(Option<&str>, Option<&str>)>,
    separated_list_complete!(
        ws!(tag!(",")),
        do_parse!(
            seen: alt!(
                map!( map_res!(is_not!(":,\""), from_utf8), split_at_last_at)
                | map!( map_res!(delimited!(char!('"'), is_not!(":,\""), char!('"')), from_utf8), split_at_last_at)
            )
            >> (seen)
        )
    )
}



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
        assert_parser!(version_line(r#"version 3.0.0"#),   Version::parse("3.0.0").unwrap());
        assert_parser!(version_line(r#"version "3.0.0""#), Version::parse("3.0.0").unwrap());
    }

    #[test]
    fn parses_head_lines() {
        let p0 = headline_parts(r#""@ava/babel-plugin-throws-helper@^2.0.0""#);
        let p1 = headline_parts(r#"assertion-error@^1.0.1, assertion-error@^1.0.1"#);
        let p2 = headline_parts(r#""@protobufjs/aspromise@^1.1.1","@protobufjs/aspromise@^1.1.2""#);
        println!("{:#?}", (p0, p1, p2));
    }

    #[test]
    fn parses_dependency_lines() {
        let p0 = dependency_line(r#"version "1.4.0""#);
        let p1 = dependency_line(r#"camelcase "^1.0.2""#);
        let p2 = dependency_line(r#"cliui "^2.1.0""#);
        let p3 = dependency_line(r#"decamelize "^1.0.0""#);
        let p4 = dependency_line(r#"window-size "0.1.0""#);

        println!("{:#?}", (p0, p1, p2, p3, p4));
    }

    #[test]
    fn print() {
        let file = test_file();
        let parsed = parse(file);
        println!("{:#?}", parsed);
    }
}
