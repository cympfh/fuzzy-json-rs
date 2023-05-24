pub mod json;
pub mod parser;

/// Fuzzy-JSON to JSON
pub fn fson(input: &str) -> Option<String> {
    parser::parse_fson(input).map(|data| data.stringify())
}
