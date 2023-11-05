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
    current_char: Option<char>
}

impl<> Tokenizer<'_> {
    pub fn new(input: Chars) -> Tokenizer {
        Tokenizer { source: input, current_col: 0, current_line: 1, token_start_col: 0, current_char: None }
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

        self.current_char = next;
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
                    self.tokenize_number(true)
                },
                '"' => {
                    self.token_start_col = self.current_col;
                    self.tokenize_string()
                },
                '-' => {
                    self.token_start_col = self.current_col;
                    if let Some(_next) = self.next_char() {
                        self.tokenize_number(false)
                    } else {
                        // TODO: Syntax error, minus must be followed by a number.
                        None
                    }
                }
                _ => {
                    // TODO: Make some actually helpful errors.
                    None
                }
            }
        }
        None
    }

    fn tokenize_number(&mut self, positive: bool) -> Option<Token> {
        /* TODO: Include support for exponents. */
        if let Some(mut number) = self.handle_integer() {
            // If we encounter a dot, we know that we're dealing with a floating point number.
            let char_count = number.chars().count();
            for _ in 1..char_count {
                self.next_char();
            }
            if let Some(next_char) = self.source.clone().next() {
                if next_char != '.' && next_char != 'e' {
                    if !positive {
                        number = '-'.to_string() + number.as_str();
                    }
                    return Some(Token::new(TokenType::Integer(number), self.token_start_col));
                }

                // We encountered a dot, get the decimal part and stitch them together.
                let is_float = next_char == '.';
                self.next_char();
                if let Some(decimal_part) = self.handle_integer() {
                    if decimal_part.chars().count() == 1 {
                        // handle_integer only returned a dot.
                        return None;
                    }
                    number = number + decimal_part.as_str();
                    let char_count = decimal_part.chars().count();
                    for _ in 1..char_count {
                        self.next_char();

                    }
                    if !positive {
                        number = '-'.to_string() + number.as_str();
                    }

                    if let Some(next_char) = self.source.clone().next() {
                        if next_char != 'e' {
                            return if is_float {
                                Some(Token::new(TokenType::Float(number), self.token_start_col))
                            } else {
                                Some(Token::new(TokenType::Integer(number), self.token_start_col))
                            }
                        }

                        // We encountered an exponent, get the decimal part and stitch them together.
                        self.next_char();
                        if let Some(is_signed) = self.source.clone().next() {
                            if is_signed == '-' || is_signed == '+' {
                                number.push('e');
                                self.next_char();
                            }
                        }
                        if let Some(decimal_part) = self.handle_integer() {
                            number = number + decimal_part.as_str();
                            let char_count = decimal_part.chars().count();
                            for _ in 1..char_count {
                                self.next_char();

                            }
                            return if is_float {
                                Some(Token::new(TokenType::Float(number), self.token_start_col))
                            } else {
                                Some(Token::new(TokenType::Integer(number), self.token_start_col))
                            }
                        }

                    } else {
                        return None;
                    }

                    return Some(Token::new(TokenType::Float(number), self.token_start_col));
                }

            } else {
                return None;
            }
        }

        return None;
    }

    fn handle_integer(&mut self) -> Option<String> {
        let first_digit: char;
        if let Some(digit) = self.current_char {
            first_digit = digit;
        } else {
            return None;
        }

        let mut result = String::from(first_digit);
        let mut iter = self.source.clone();
        while let Some(digit) = iter.next() {
            match digit {
                '0'..='9' => {
                    result.push(digit);
                }
                _ => {
                    return Some(result)
                }
            }
        }

        Some(result)
    }

    fn tokenize_string(&mut self) -> Option<Token> {
        let mut string_val = String::new();

        let mut iter = self.source.clone();
        let mut skip = 0;
        while let Some(next_char) = iter.next() {
            if next_char.is_control() {
                return None;
            }

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
            Token::new(TokenType::Integer("2".to_string()), 20),
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
        let complete_string = r#""This string is completed and should be tokenized.""#;
        let mut lexer = Tokenizer::new(complete_string.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::String("This string is completed and should be tokenized.".to_string()), 1),
        ];
        assert_eq!(tokens, expected_tokens);

        let incomplete_string = r#""This string is missing a quotation mark at the end and should not be tokenized."#;
        let mut lexer = Tokenizer::new(incomplete_string.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::String("This string is missing a quotation mark at the end and should not be tokenized.}".to_string()), 1),
        ];
        assert_ne!(tokens, expected_tokens);

        let money_is_fire = r#"{"money": "💶=🔥"}"#;
        let mut lexer = Tokenizer::new(money_is_fire.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ObjectStart, 1),
            Token::new(TokenType::String("money".to_string()), 2),
            Token::new(TokenType::Colon, 9),
            Token::new(TokenType::String("💶=🔥".to_string()), 11),
            Token::new(TokenType::ObjectEnd, 16)
        ];
        assert_eq!(tokens, expected_tokens);

        let ctrlseq = r#"{"ctrlseq": "
        "}"#;
        let mut lexer = Tokenizer::new(ctrlseq.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ObjectStart, 1),
            Token::new(TokenType::String("ctrlseq".to_string()), 2),
            Token::new(TokenType::Colon, 11),
            Token::new(TokenType::String("💶=🔥".to_string()), 13),
            Token::new(TokenType::ObjectEnd, 16)
        ];
        assert_ne!(tokens, expected_tokens);
    }

    #[test]
    fn numbers() {
        let integer = r#"[5, -10, -928472]"#;
        let mut lexer = Tokenizer::new(integer.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ArrayStart, 1),
            Token::new(TokenType::Integer("5".to_string()), 2),
            Token::new(TokenType::Comma, 3),
            Token::new(TokenType::Integer("-10".to_string()), 5),
            Token::new(TokenType::Comma, 8),
            Token::new(TokenType::Integer("-928472".to_string()), 10),
            Token::new(TokenType::ArrayEnd, 17),
        ];
        assert_eq!(tokens, expected_tokens);

        let float = r#"[5.23, -23.0923787687]"#;
        let mut lexer = Tokenizer::new(float.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ArrayStart, 1),
            Token::new(TokenType::Float("5.23".to_string()), 2),
            Token::new(TokenType::Comma, 6),
            Token::new(TokenType::Float("-23.0923787687".to_string()), 8),
            Token::new(TokenType::ArrayEnd, 22),
        ];
        assert_eq!(tokens, expected_tokens);

        let exponents = r#"[1e2, 1.0e2, 2.0879878e243, -32.928e-54, -32.928e+54]"#;
        let mut lexer = Tokenizer::new(exponents.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![
            Token::new(TokenType::ArrayStart, 1),
            Token::new(TokenType::Integer("1e2".to_string()), 2),
            Token::new(TokenType::Comma, 5),
            Token::new(TokenType::Float("1.0e2".to_string()), 7),
            Token::new(TokenType::Comma, 12),
            Token::new(TokenType::Float("2.0879878e243".to_string()), 14),
            Token::new(TokenType::Comma, 27),
            Token::new(TokenType::Float("-32.928e-54".to_string()), 29),
            Token::new(TokenType::Comma, 40),
            Token::new(TokenType::Float("-32.928e+54".to_string()), 42),
            Token::new(TokenType::ArrayEnd, 53),
        ];
        assert_eq!(tokens, expected_tokens);

        let do_not_recognize = r#"5."#;
        let mut lexer = Tokenizer::new(do_not_recognize.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![];
        assert_eq!(tokens, expected_tokens);

        let do_not_recognize = r#"+5"#;
        let mut lexer = Tokenizer::new(do_not_recognize.chars());
        let tokens = lexer.tokenize();
        let expected_tokens = vec![];
        assert_eq!(tokens, expected_tokens);
    }
}
