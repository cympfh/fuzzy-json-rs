extern crate json;
use json::stringify;

#[derive(Debug, Clone, PartialEq)]
pub enum JSON {
    Int(i128),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<JSON>),
    Dict(Vec<(String, JSON)>),
    Null,
}

impl JSON {
    pub fn stringify(&self) -> String {
        use JSON::*;
        match self {
            Int(x) => format!("{}", x),
            Float(x) => format!("{}", x),
            Bool(x) => format!("{:?}", x),
            Str(x) => stringify(x.to_string()),
            Array(xs) => format!(
                "[{}]",
                xs.iter()
                    .map(|j| j.stringify())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Dict(d) => format!(
                "{{{}}}",
                d.iter()
                    .map(|(key, val)| format!("\"{}\":{}", key, val.stringify()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Null => "null".to_string(),
        }
    }
}
