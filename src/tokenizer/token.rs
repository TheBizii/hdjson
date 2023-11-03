#[derive(Debug, PartialEq)]
pub enum TokenType {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Comma,
    Colon,
    Number(u32),
    String(String)
}

#[derive(Debug, PartialEq)]
pub struct Token {
    token_type: TokenType,
    position: i32,
}

impl Token {
    pub fn new(token_type: TokenType, position: i32) -> Token {
        Token {
            token_type,
            position,
        }
    }
}
