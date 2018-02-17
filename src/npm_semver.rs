use semver_parser::range::{Op, Predicate, VersionReq as VersionReqInner, WildcardVersion};
use semver::VersionReq;
use nom::IResult;

use std::str::FromStr;

type Semver<'a> = (Op, u64, Option<&'a str>, Option<&'a str>);

/// Tuple of a lower and optinal upper version limit
type SemverRange<'a> = (Semver<'a>, Option<Semver<'a>>);

pub fn version_reqs(raw: &[u8]) -> IResult<&[u8], Vec<VersionReq>> {
    parsers::semver_range_list(raw)
           .map(|r| r.into_iter()
                     .map(range2req)
                     .map(Into::into)
                     .collect()
    )
}

 fn range2req(range: SemverRange) -> VersionReqInner {
     VersionReqInner {
         predicates: Some(range.0).into_iter()
             .chain(range.1.into_iter())
             .map(|vers| {
                 let op = match (vers.0, vers.2) {
                     (_, Some("x")) => Op::Wildcard(WildcardVersion::Minor),
                     (x, _) => x
                 };

                 Predicate {
                    op,
                    major: vers.1,
                    minor: vers.2.and_then(|x| FromStr::from_str(x).ok()),
                    patch: vers.3.and_then(|x| FromStr::from_str(x).ok()),
                    pre: Vec::new(),
                }
             })
             .collect::<Vec<Predicate>>()
      }
 }

mod parsers {
    use super::*;

    use nom::{is_alphanumeric, rest};
    use std::str::from_utf8;
    use std::str::FromStr;
    use semver_parser::range::Op;

    named!{ take_allowed(&[u8]) -> &[u8],

        take_while!(is_alphanumeric)
    }

    named!{ tilldot(&[u8]) -> u64,
        map_res!(
            map_res!(take_allowed, from_utf8),
            FromStr::from_str
            )
    }

    named!{ fromdot(&[u8]) -> &str,

        complete!(
        do_parse!(
            tag!(".") >>
            string: map_res!(take_allowed, from_utf8) >>

            (string)
        )
        )
    }

    named!{ semver_prefix(&[u8]) -> &str,
        map_res!(
            alt!(
                tag!("<=") | tag!("<") |
                tag!(">=") | tag!(">") |
                tag!("^")  | tag!("~") | tag!("=")
            ), from_utf8
        )
    }

    named!{ semver_op(&[u8]) -> Op,
        map!(
            opt!(semver_prefix)
            , prefix_to_op
        )
    }

    fn prefix_to_op(prefix: Option<&str>) -> Op {
        match prefix {
            Some("=")  => Op::Ex,
            Some(">")  => Op::Gt,
            Some(">=") => Op::GtEq,
            Some("<")  => Op::Lt,
            Some("<=") => Op::LtEq,
            Some("~")  => Op::Tilde,
            Some("^")  => Op::Compatible,
            None       => Op::Ex,
            Some(_)    => unreachable!()
        }
    }

    named!{ tagged_semver(&[u8]) -> (Semver, &str),

        do_parse!(
            semver: semver >>
            opt!(tag!("-")) >>
            rest: map_res!(rest, from_utf8) >>
            (semver, rest)
        )
    }

    named!{pub semver(&[u8]) -> Semver,

        ws!(
        do_parse!(
            prefix: semver_op    >>
            major: tilldot       >>
            minor: opt!(fromdot) >>
            patch: opt!(fromdot) >>
            (prefix, major, minor, patch)
        )
        )
    }


    named!{pub semver_range(&[u8]) -> SemverRange,

        ws!(
        do_parse!(
            from: semver >>
            //tap!(opt!(alt!(tag!("<=") | tag!("<")))) >>
            till: opt!(semver) >>
            (from, till)
        )
        )
    }

    named!{pub semver_range_list(&[u8]) -> Vec<SemverRange>,

        delimited!(
            char!('"'),
            separated_list_complete!(ws!(tag!("||")), semver_range),
            char!('"'))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        macro_rules! assert_parser {
            ($parser: expr, $expected: expr) => {
                assert_eq!($parser, IResult::Done(&[][..], $expected));
            };
        }

        #[test]
        fn parse_semver() {
            assert_parser!(semver(b"2.3.4"),        (Op::Ex,         2, Some("3"), Some("4")));
            assert_parser!(semver(b"<=2.3.4"),      (Op::LtEq,       2, Some("3"), Some("4")));
            assert_parser!(semver(b">=2.3.4"),      (Op::GtEq,       2, Some("3"), Some("4")));
            assert_parser!(semver(b"=2.3.4"),       (Op::Ex,         2, Some("3"), Some("4")));
            assert_parser!(semver(b">2.3.4"),       (Op::Gt,         2, Some("3"), Some("4")));
            assert_parser!(semver(b"^2.3.4"),       (Op::Compatible, 2, Some("3"), Some("4")));
            assert_parser!(semver(b"^ 2.3.4"),      (Op::Compatible, 2, Some("3"), Some("4")));
            assert_parser!(semver(b"~2.3.4"),       (Op::Tilde,      2, Some("3"), Some("4")));
            assert_parser!(semver(b"~ 2.3.4"),      (Op::Tilde,      2, Some("3"), Some("4")));
            assert_parser!(semver(b" >= 2.3.4"),    (Op::GtEq,       2, Some("3"), Some("4")));
            assert_parser!(semver(b"2.3.4"),        (Op::Ex,         2, Some("3"), Some("4")));
            assert_parser!(semver(b"2.3.4"),        (Op::Ex,         2, Some("3"), Some("4")));

            assert_parser!(semver(b">= 2.3"),       (Op::GtEq,       2, Some("3"), None)     );
            assert_parser!(semver(b"2.x"),          (Op::Ex,         2, Some("x"), None)     );

            assert_parser!(semver(b"2.3"),          (Op::Ex,         2, Some("3"), None)     );
            assert_parser!(semver(b"2"),            (Op::Ex,         2, None,      None)     );
            assert_parser!(semver(b" 2.3 "),        (Op::Ex,         2, Some("3"), None)     );
            assert_parser!(semver(b" 2 "),          (Op::Ex,         2, None,      None)     );
        }

        #[test]
        fn parse_semver_range() {
            assert_parser!(
                semver_range(b">=2.3.4 < 3"), (
                    (Op::GtEq, 2, Some("3"), Some("4")),
               Some((Op::Lt,   3, None, None))
                    )
                );

            let expected = (
                    (Op::GtEq, 2, Some("3"), Some("4")),
               Some((Op::Lt,   3, Some("2"), None))
                    );

            assert_parser!( semver_range(b">=2.3.4 < 3.2"), expected);

            assert_parser!( semver_range(b">=2.3.4<3.2.1"),
                ((Op::GtEq, 2, Some("3"), Some("4")), Some((Op::Lt, 3, Some("2"), Some("1")))));

            assert_parser!( semver_range(b">=2.3.4 < 3.2.1"),
                ((Op::GtEq, 2, Some("3"), Some("4")), Some((Op::Lt, 3, Some("2"), Some("1")))));


        }

        #[test]
        fn parse_tagged_semver() {
            assert_parser!(
                tagged_semver(b"2.3.4-pre"),
                ((Op::Ex, 2, Some("3"), Some("4")), "pre")
                );
        }

        #[test]
        fn parse_npm_semver_range_list() {
            let samples = [
                r#"">=2.2.7 <3""#,
                r#""^2.0.0 || ^1.1.13""#,
                r#""> 1.0.0 < 3.0.0""#,
                r#"">=0.3.0 <0.4""#,
                r#""1 || 2""#,
                r#"">=0.5 0""#,
                r#""2 || 3""#,
                r#"">= 1.3.1 < 2""#,
                r#""^4.0.0 || ^5.0.0""#,
                r#""1 || 2""#,
                r#""2.x || 3.x || 4 || 5""#,
                r#""2 || 3""#,
                r#""0 || 1 || 2 || 3 || 4""#,
                r#""2 || 3 || 4 || 5""#,
                r#""^2.3.0 || 3.x || 4 || 5""#,
                r#""~1.0.1 || ^2.0.0""#,
                r#""^3.0.0 || ^4.0.0 || ^5.0.0""#,
                r#""2 >=2.2.1 || 3.x || 4 || 5""#,
                r#""2 || 3 || 4""#,
                r#"">= 0.0.5 < 1""#,
                r#"">= 1.0.2 < 2""#,
                r#"">= 1.0.0 < 2""#,
                r#""2 || 3 || 4 || 5""#,
                r#"">=0.10.3 <1""#,
                r#"">= 3.1.0 < 4""#,
                r#"">= 0.3.0 < 1""#,
                r#"">= 0.1.30 < 1""#,
                r#"">= 0.0.2 < 1""#,
                r#"">= 0.4.1 < 1""#,
                r#"">= 1.0.1 < 2""#,
            ];
            for sample in &samples {
                println!("{:?}", semver_range_list(sample.as_bytes()).unwrap());
            }
    }
}

}