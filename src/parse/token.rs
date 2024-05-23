#[derive(Debug, Clone)]
pub struct Token { 
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: i32,
    pub id: u32, // used for resolving names
}


#[derive(Debug, Clone)]
pub enum TokenType {
    EOF,
    Comma,
    Semicolon,
    Select,
    From,
    Identifier,
    False,
    True,
    Null,
    Int,
    IntLiteral,
    Insert,
    Into,
    Values,
    Bool,
    Create,
    Table,
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    IntValue(i64),
    StringValue(String),
    BooleanValue(bool),
    NullValue
}