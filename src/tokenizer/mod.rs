//! Module for performing tokenization of JSON inputs.

mod json_value;
mod token;

use std::str::Chars;
pub use crate::tokenizer::token::Token;
pub use crate::tokenizer::token::TokenType;

pub struct Tokenizer<'a> {
    source: Chars<'a>,
    current_col: i32,
    current_line: i32,
    token_start_col: i32,
}

impl<> Tokenizer<'_> {
    pub fn new(input: Chars) -> Tokenizer {
        Tokenizer { source: input, current_col: 0, current_line: 1, token_start_col: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }

    pub fn next_char(&mut self) -> Option<char> {
        self.current_col += 1;
        let next = self.source.next();
        if let Some(ch) = next {
            if ch == '\n' {
                self.current_line += 1;
            }
        }

        next
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let next_char = self.skip_whitespaces();
        if let Some(ch) = next_char {
            return match ch {
                '{' => Some(Token::new(TokenType::ObjectStart, self.current_col)),
                '}' => Some(Token::new(TokenType::ObjectEnd, self.current_col)),
                '[' => Some(Token::new(TokenType::ArrayStart, self.current_col)),
                ']' => Some(Token::new(TokenType::ArrayEnd, self.current_col)),
                ':' => Some(Token::new(TokenType::Colon, self.current_col)),
                ',' => Some(Token::new(TokenType::Comma, self.current_col)),
                '0'..='9' => {
                    self.token_start_col = self.current_col;
                    self.tokenize_number(ch)
                },
                '"' => {
                    self.token_start_col = self.current_col;
                    self.tokenize_string()
                },
                // TODO: Handle whitespaces and newlines
                // TODO: Handle signed numbers (signs: none, +, -)
                _ => {
                    // TODO: Make some actually helpful errors.
                    None
                }
            }
        }
        None
    }

    fn tokenize_number(&mut self, first_digit: char) -> Option<Token> {
        /* TODO: This handles only positive integers. Include support for fractions and
            exponents. */
        if let Some(mut number) = first_digit.to_digit(10) {
            while let Some(next_char) = self.source.clone().next() {
                if let Some(digit) = next_char.to_digit(10) {
                    number = number * 10 + digit;
                    self.next_char();
                } else {
                    return Some(Token::new(TokenType::Number(number), self.token_start_col));
                }
            }
            Some(Token::new(TokenType::Number(number),self.token_start_col));
        }
        None
    }

    fn tokenize_string(&mut self) -> Option<Token> {
        /* TODO: Strings should not contain \n control characters. Throw an error. */
        let mut string_val = String::new();

        let mut iter = self.source.clone();
        let mut escape_seq_found = false;
        let mut skip = 0;
        while let Some(next_char) = iter.next() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if next_char == '\\' {
                self.next_char();
                if let Some(escape) = self.handle_escapes() {
                    string_val += escape.as_str();
                    // Some iterations will need to be skipped, because this loop is using a
                    // cloned iterator that, handle_escapes() operates on the original one.
                    skip = escape.chars().count() - 1;
                } else {
                    // TODO: Display error.
                }
                escape_seq_found = false;
                continue;
            }

            if next_char != '"' {
                string_val.push(next_char);
                self.next_char();
            } else {
                break;
            }
        }
        if let Some(next_char) = self.next_char() {
            // String values must end with a " quotation mark.
            return if next_char == '\"' {
                Some(Token::new(TokenType::String(string_val.clone()), self.token_start_col))
            } else {
                None
            }
        }
        None
    }

    fn handle_escapes(&mut self) -> Option<String> {
        if let Some(next) = self.next_char() {
            let mut escape = String::from("\\");
            escape.push(next);
            return match next {
                '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {
                    Some(escape)
                },
                'u' => {
                    for _ in 0..4 {
                        if let Some(seq_char) = self.next_char() {
                            if !seq_char.is_ascii_hexdigit() {
                                return None;
                            }
                            escape.push(seq_char);
                        } else {
                            return None;
                        }
                    }
                    Some(escape)
                },
                _ => {
                    // TODO: Error handling, illegal escape sequence.
                    None
                }
            }
        }
        None
    }

    fn skip_whitespaces(&mut self) -> Option<char> {
        while let Some(ch) = self.next_char() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    continue;
                }
                _ => {
                    return Some(ch);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{Tokenizer, Token, TokenType};

    #[test]
    fn simple_objects() {
        let json_str = r#"{"coolness_factor":2,"description":"This is kinda \"cool\"!"}"#;
        let mut lexer = Tokenizer::new(json_str.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ObjectStart, 1),
            Token::new(TokenType::String("coolness_factor".to_string()), 2),
            Token::new(TokenType::Colon, 19),
            Token::new(TokenType::Number(2), 20),
            Token::new(TokenType::Comma, 21),
            Token::new(TokenType::String("description".to_string()), 22),
            Token::new(TokenType::Colon, 35),
            Token::new(TokenType::String("This is kinda \\\"cool\\\"!".to_string()), 36),
            Token::new(TokenType::ObjectEnd, 61)
        ];
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn escape_sequences() {
        let json_str = r#"{"allowed":"\u0009","allowed1":"\b","allowed2":"\n","allowed3":"\\"}"#;
        let mut lexer = Tokenizer::new(json_str.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ObjectStart, 1),
            Token::new(TokenType::String("allowed".to_string()), 2),
            Token::new(TokenType::Colon, 11),
            Token::new(TokenType::String("\\u0009".to_string()), 12),
            Token::new(TokenType::Comma, 20),
            Token::new(TokenType::String("allowed1".to_string()), 21),
            Token::new(TokenType::Colon, 31),
            Token::new(TokenType::String("\\b".to_string()), 32),
            Token::new(TokenType::Comma, 36),
            Token::new(TokenType::String("allowed2".to_string()), 37),
            Token::new(TokenType::Colon, 47),
            Token::new(TokenType::String("\\n".to_string()), 48),
            Token::new(TokenType::Comma, 52),
            Token::new(TokenType::String("allowed3".to_string()), 53),
            Token::new(TokenType::Colon, 63),
            Token::new(TokenType::String("\\\\".to_string()), 64),
            Token::new(TokenType::ObjectEnd, 68)
        ];
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn strings() {
        let complete_string = r#"{"string": "This string is completed and should be tokenized."}"#;
        let mut lexer = Tokenizer::new(complete_string.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ObjectStart, 1),
            Token::new(TokenType::String("string".to_string()), 2),
            Token::new(TokenType::Colon, 10),
            Token::new(TokenType::String("This string is completed and should be tokenized.".to_string()), 12),
            Token::new(TokenType::ObjectEnd, 63)
        ];
        assert_eq!(tokens, expected_tokens);

        let incomplete_string = r#"{"invalid": "This string is missing a quotation mark at the end and should not be tokenized.}"#;
        let mut lexer = Tokenizer::new(incomplete_string.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ObjectStart, 1),
            Token::new(TokenType::String("invalid".to_string()), 2),
            Token::new(TokenType::Colon, 11),
            Token::new(TokenType::String("This string is missing a quotation mark at the end and should not be tokenized.}".to_string()), 13),
        ];
        assert_ne!(tokens, expected_tokens);
    }
}
