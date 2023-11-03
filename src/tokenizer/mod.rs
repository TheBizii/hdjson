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
        let next_char = self.next_char();
        if let Some(ch) = next_char {
            return match ch {
                '{' => Some(Token::new(TokenType::ObjectStart, self.current_col)),
                '}' => Some(Token::new(TokenType::ObjectEnd, self.current_col)),
                '[' => Some(Token::new(TokenType::ArrayStart, self.current_col)),
                ']' => Some(Token::new(TokenType::ArrayEnd, self.current_col)),
                ':' => Some(Token::new(TokenType::Colon, self.current_col)),
                ',' => Some(Token::new(TokenType::Comma, self.current_col)),
                '0'..='9' => {
                    /* TODO: This handles only positive integers. Include support for fractions and
                        exponents. Also, create a separate function, this is getting messy. */
                    self.token_start_col = self.current_col;
                    if let Some(mut number) = ch.to_digit(10) {
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
                },
                '"' => {
                    /* TODO: This should be a separate function. Also, improve handling of
                        escape sequences. */
                    self.token_start_col = self.current_col;
                    let mut string_val = String::new();
                    let mut escape_seq_found = false;

                    while let Some(next_char) = self.source.clone().next() {
                        if next_char == '\\' {
                            escape_seq_found = true;
                        }

                        if next_char != '"' {
                            string_val.push(next_char);
                            self.next_char();
                        } else {
                            if escape_seq_found {
                                string_val.push(next_char);
                                self.next_char();
                                escape_seq_found = false;
                            } else {
                                Some(Token::new(TokenType::String(string_val.clone()), self.token_start_col));
                                break;
                            }
                        }
                    }
                    self.next_char();
                    Some(Token::new(TokenType::String(string_val.clone()), self.token_start_col))
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
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{Tokenizer, Token, TokenType};

    #[test]
    fn tokenize_simple_objects() {
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
}
