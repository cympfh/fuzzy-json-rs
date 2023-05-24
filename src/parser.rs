use nom::combinator;
use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_while, take_while1},
    character::complete::{char, one_of},
    combinator::{eof, map, opt, recognize, value},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

use crate::json::JSON;
pub fn parse_fson(input: &str) -> Option<JSON> {
    let n = input.len();
    let mut input = input.chars();
    for _ in 0..n {
        if let Ok((_, data)) = parse_data(input.as_str()) {
            return Some(data);
        }
        input.next();
    }
    None
}

pub fn parse_data(input: &str) -> IResult<&str, JSON> {
    alt((
        parse_dict,
        parse_array,
        parse_null,
        parse_bool,
        parse_str,
        parse_float,
        parse_int,
    ))(input)
}

pub fn spaces(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = alt((tag("//"), tag("#"), tag(";"), tag("--")))(input)?;
    let (input, _) = opt(is_not("\n\r"))(input)?;
    alt((eof, spaces))(input)
}

pub fn commentable_spaces(input: &str) -> IResult<&str, ()> {
    let (input, _) = spaces(input)?;
    let (input, _) = many0(tuple((comment, spaces)))(input)?;
    Ok((input, ()))
}

pub fn identifier(input: &str) -> IResult<&str, String> {
    fn head(c: char) -> bool {
        c.is_alphabetic() || c == '_' || c == '#' || c == '@'
    }
    fn tail(c: char) -> bool {
        c.is_alphanumeric() || head(c)
    }
    let (input, s) = take_while1(head)(input)?;
    let (input, t) = take_while(tail)(input)?;
    let mut name = String::new();
    name.push_str(s);
    name.push_str(t);
    Ok((input, name))
}

pub fn parse_null(input: &str) -> IResult<&str, JSON> {
    value(
        JSON::Null,
        alt((
            tag("null"),
            tag("Null"),
            tag("NULL"),
            tag("NUL"),
            tag("Nil"),
            tag("nil"),
            tag("None"),
            tag("Nothing"),
        )),
    )(input)
}

pub fn parse_bool(input: &str) -> IResult<&str, JSON> {
    alt((
        value(
            JSON::Bool(true),
            alt((
                tag("true"),
                tag("True"),
                tag("TRUE"),
                tag("yes"),
                tag("Yes"),
                tag("YES"),
            )),
        ),
        value(
            JSON::Bool(false),
            alt((
                tag("false"),
                tag("False"),
                tag("FALSE"),
                tag("no"),
                tag("No"),
                tag("NO"),
            )),
        ),
    ))(input)
}

pub fn parse_int(input: &str) -> IResult<&str, JSON> {
    fn decimal(input: &str) -> IResult<&str, &str> {
        recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
    }
    let mut num_value = map(pair(opt(tag("-")), decimal), |(sign, num): (_, &str)| {
        let num: String = num.chars().filter(|&c| c != '_').collect();
        let val = num.parse::<i128>().unwrap();
        match sign {
            None => JSON::Int(val),
            _ => JSON::Int(-val),
        }
    });
    num_value(input)
}

pub fn parse_float(input: &str) -> IResult<&str, JSON> {
    fn decimal(input: &str) -> IResult<&str, &str> {
        recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
    }
    let mut value = map(
        alt((
            recognize(tuple((opt(char('-')), char('.'), decimal))),
            recognize(tuple((opt(char('-')), decimal, char('.'), decimal))),
        )),
        |num_str: &str| {
            let num: String = num_str.chars().filter(|&c| c != '_').collect();
            let x: f64 = num.parse().unwrap();
            JSON::Float(x)
        },
    );
    value(input)
}

pub fn parse_str(input: &str) -> IResult<&str, JSON> {
    let mut value = alt((
        combinator::value(JSON::Str(String::new()), tag("\"\"")),
        map(
            delimited(
                tag("\""),
                escaped_transform(
                    is_not("\"\\"),
                    '\\',
                    alt((
                        combinator::value("\\", tag("\\")),
                        combinator::value("\"", tag("\"")),
                        combinator::value("\'", tag("\'")),
                        combinator::value("\n", tag("n")),
                        combinator::value("\r", tag("r")),
                        combinator::value("\t", tag("t")),
                    )),
                ),
                tag("\""),
            ),
            JSON::Str,
        ),
        combinator::value(JSON::Str(String::new()), tag("\'\'")),
        map(
            delimited(
                tag("\'"),
                escaped_transform(
                    is_not("\'\\"),
                    '\\',
                    alt((
                        combinator::value("\\", tag("\\")),
                        combinator::value("\"", tag("\"")),
                        combinator::value("\'", tag("\'")),
                        combinator::value("\n", tag("n")),
                        combinator::value("\r", tag("r")),
                        combinator::value("\t", tag("t")),
                    )),
                ),
                tag("\'"),
            ),
            JSON::Str,
        ),
    ));
    value(input)
}

pub fn parse_array(input: &str) -> IResult<&str, JSON> {
    let mut value = map(
        tuple((
            tag("["),
            commentable_spaces,
            separated_list0(
                tuple((tag(","), commentable_spaces)),
                terminated(parse_data, commentable_spaces),
            ),
            opt(tuple((tag(","), commentable_spaces))),
            tag("]"),
        )),
        |(_, _, elems, _, _)| JSON::Array(elems),
    );
    value(input)
}

pub fn parse_dict(input: &str) -> IResult<&str, JSON> {
    let mut value = map(
        tuple((
            tag("{"),
            commentable_spaces,
            separated_list0(
                tuple((tag(","), commentable_spaces)),
                map(
                    tuple((
                        alt((
                            map(parse_str, |data| match data {
                                JSON::Str(s) => s,
                                _ => panic!(),
                            }),
                            identifier,
                        )),
                        commentable_spaces,
                        tag(":"),
                        commentable_spaces,
                        parse_data,
                        commentable_spaces,
                    )),
                    |(name, _, _, _, data, _)| (name, data),
                ),
            ),
            opt(tuple((tag(","), commentable_spaces))),
            tag("}"),
        )),
        |(_, _, items, _, _)| JSON::Dict(items),
    );
    value(input)
}

#[cfg(test)]
mod test_parser {

    use crate::json::JSON;
    use crate::parser::parse_fson;

    macro_rules! assert_value {
        ($code: expr, $expected: expr) => {
            assert_eq!(
                parse_fson($code),
                Some($expected),
                "{} != {:?}",
                $code,
                $expected
            );
        };
    }

    #[test]
    fn test_simple() {
        assert_value!("null", JSON::Null);
        assert_value!("None", JSON::Null);
        assert_value!("NUL.", JSON::Null);
        assert_value!("yes.", JSON::Bool(true));
        assert_value!("NO NO.", JSON::Bool(false));
        assert_value!("This is a NULL.", JSON::Null);
        assert_value!("PI is 3.141", JSON::Float(3.141));
        assert_value!("PI is 3.", JSON::Int(3));
        assert_value!("\"\"", JSON::Str(String::from("")));
        assert_value!("''", JSON::Str(String::from("")));
        assert_value!("\"dog\"", JSON::Str(String::from("dog")));
        assert_value!("\"'\"", JSON::Str(String::from("'")));
        assert_value!("'\"'", JSON::Str(String::from("\"")));
    }

    #[test]
    fn test_array() {
        assert_value!("[]", JSON::Array(vec![]));
        assert_value!("[null]", JSON::Array(vec![JSON::Null]));
        assert_value!(
            "[null, 1, []]",
            JSON::Array(vec![JSON::Null, JSON::Int(1), JSON::Array(vec![]),])
        );
    }

    #[test]
    fn test_dict() {
        assert_value!("{}", JSON::Dict(vec![]));
        assert_value!(
            "{x: 42, \"x 2\": {}}",
            JSON::Dict(vec![
                (String::from("x"), JSON::Int(42)),
                (String::from("x 2"), JSON::Dict(vec![])),
            ])
        );
    }

    #[test]
    fn test_complex() {
        assert_value!(
            r#"
{
  "x": 42,
  'y': {},
  z: [ nil ], // trailing comma!
}.
            "#,
            JSON::Dict(vec![
                (String::from("x"), JSON::Int(42)),
                (String::from("y"), JSON::Dict(vec![])),
                (String::from("z"), JSON::Array(vec![JSON::Null])),
            ])
        );
    }
}
