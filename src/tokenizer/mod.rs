mod json_value;
mod lexeme;

pub use self::json_value::JsonValue;

pub struct Lexer {}

impl Lexer {
    pub fn tokenize(_input: &str) -> Vec<JsonValue> {
        vec![JsonValue::Null]
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{JsonValue, Lexer};

    #[test]
    fn tokenize_simple_objects() {
        let json_str = r#"{"description": "Super duper new Rust library"}"#;
        let parsed = Lexer::tokenize(json_str);
        let expected = vec![JsonValue::Null];
        assert_eq!(parsed, expected);
    }
}
