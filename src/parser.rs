
#![allow(unused_parens)]
use super::DependencyLock;
use indent_tokenizer::{self, tokenize, Token};
use nom::{line_ending, IResult};
use std::collections::HashMap;
use std::str::from_utf8;

fn read_level2(tokens: &[Token]) -> HashMap<String, String> {
    tokens
        .iter()
        .flat_map(|t| &t.lines)
        .filter(|ref l| !l.starts_with("dependencies"))
        .filter_map(|level2| {
            let tup_line = tuple_line(level2);
            if let IResult::Done(_left_overs, tup_line) = tup_line {
                let (key, val) = tup_line;
                Some((key.to_string(), val.to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn read_dependencies(tokens: &[Token]) -> HashMap<String, String> {
    tokens.iter()
        .filter(|token| token.lines.last().map(|s| s.starts_with("dependencies")).unwrap_or(false))
        .flat_map(|t| &t.tokens)
        .nth(0).iter()
        .flat_map(|t| &t.lines)
        //.map(|sline| String::from(sline.as_ref()))
        .map(|sline| sline)
            .filter_map(|level2| {
                let tup_line = tuple_line(level2);
                if let IResult::Done(_left_overs, tup_line) = tup_line {
                    let (key, val) = tup_line;
                    Some((key.to_string(), val.to_string()))
                } else {
                    None
                }
            })
        .collect()
}

fn read_block(block: &Token) -> Vec<DependencyLock> {
    let properties = read_level2(&block.tokens);
    let dependencies = read_dependencies(&block.tokens);

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
                    last_seen:    last_seen.unwrap().to_string(),
                    properties:   properties.clone(),
                    dependencies: dependencies.clone(),
                })
        })
        .collect()
}

pub fn parse(content: &str) -> Result<Vec<DependencyLock>, Box<indent_tokenizer::Error>> {
    Ok(tokenize(content)?.iter().flat_map(read_block).collect())
}

pub fn headline_parts(content: &str) -> IResult<&[u8], Vec<(Option<&str>, Option<&str>)>> {
    headline_parts_int(content.as_bytes())
}

named!(pub headline(&[u8]) -> &str,
        do_parse!(
            content: map_res!(take_till_s!(|c| c == ':' as u8), from_utf8) >>
            opt!(line_ending) >>
            (content)
        )
    );

fn split_at_last_at(s: &str) -> (Option<&str>, Option<&str>) {
    let mut it = s.rsplitn(2, '@');
    (it.nth(0), it.nth(0))
}

named!(headline_parts_int(&[u8]) -> Vec<(Option<&str>, Option<&str>)>,
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
    );

pub fn tuple_line(content: &str) -> IResult<&[u8], (&str, &str)> {
    tuple_line_int(content.as_bytes())
}

named!(tuple_line_int(&[u8]) -> (&str, &str),
        ws!(tuple!(alt!(
                     map_res!(delimited!(char!('"'), is_not!(",\""), char!('"')), from_utf8)
                   | map_res!(is_not!(" "), from_utf8)
                    ),
            map_res!(delimited!(char!('"'), is_not!(",\""), char!('"')), from_utf8)
        ))
    );
