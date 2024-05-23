use std::{iter::Peekable, str::Chars};

use super::token::{LiteralValue, Token, TokenType};


pub struct Scanner {
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: i32,
    next_id: u32,
}

#[derive(Debug)]
pub enum TError {
    ScanError(String),
    ParseError(String),
}

impl<'a> Scanner {
    pub fn new() -> Scanner {
        Scanner { tokens: Vec::<Token>::new(), start: 0, current: 0, line: 1, next_id: 0 }
    }

    pub fn scan(&mut self, source: &'a String) -> Result<(), TError> {
        self.tokens.clear();
        self.start = 0;
        self.current = 0;
        self.line = 1;

        let mut chars = source.chars().peekable();

        loop {
            if chars.peek().is_none() {
                break;
            }
            self.scan_token(&mut chars, source)?;
            self.start = self.current;
        }
        self.add_token(TokenType::EOF, "".to_owned(), None);

        Ok(())
    }
    
    fn scan_token(&mut self, chars: &mut Peekable<Chars<'_>>, source: &'a String) -> Result<(), TError> {
        let s = self.advance(chars).unwrap();
        match s {
            ',' => self.add_simple_token(TokenType::Comma, source),
            ';' => self.add_simple_token(TokenType::Semicolon, source),
            '(' => self.add_simple_token(TokenType::LeftParen, source),
            ')' => self.add_simple_token(TokenType::RightParen, source),

            ' ' | '\t' | '\r' | '\n' => {},

            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                return self.scan_number(chars, source)
            }

            _ => {
                if s.is_alphabetic() {
                    return self.scan_alphabetic(chars, source);
                } else {
                    return Err(TError::ScanError(format!("Unrecognized symbol {}", s)));
                }
            }
        };
        Ok(())
    }

    fn add_simple_token(&mut self, token_type: TokenType, source: &'a String) {
        let lexeme = &source[self.start..self.current];
        self.add_token(token_type, lexeme.to_owned(), None);
    }

    fn advance(&mut self, chars: &mut Peekable<Chars<'_>>) -> Option<char> {
        self.current += 1;
        chars.next()
    }
    
    fn add_token(&mut self, token_type: TokenType, lexeme: String, literal: Option<LiteralValue>) {
        let t = Token {token_type, lexeme, literal, line: self.line, id: self.next_id};
        self.next_id += 1;
        self.tokens.push(t);
    }

    fn scan_alphabetic(&mut self, chars: &mut Peekable<Chars<'_>>, source: &'a String) -> Result<(), TError> {
        loop {
            if let Some(possible_alphabetic) = chars.peek() {
                if possible_alphabetic.is_alphanumeric() {
                    self.advance(chars);
                } else {
                    break;
                }
            } else {
                break;
            }
        };
        let lexeme = &source[self.start..self.current];
        let token_type = match lexeme {
            "select" => TokenType::Select,
            "from" => TokenType::From,
            "insert" => TokenType::Insert,
            "into" => TokenType::Into,
            "values" => TokenType::Values,
            "create" => TokenType::Create,
            "table" => TokenType::Table,

            "int" => TokenType::Int,
            "bool" => TokenType::Bool,

            "false" => TokenType::False,
            "true" => TokenType::True,
            "null" => TokenType::Null,
            
            _ => TokenType::Identifier
        };
        let literal = match &token_type {
            TokenType::False => Some(LiteralValue::BooleanValue(false)),
            TokenType::True => Some(LiteralValue::BooleanValue(true)),
            TokenType::Null => Some(LiteralValue::NullValue),
            _ => None
        };
        self.add_token(token_type, lexeme.to_owned(), literal);
        Ok(())
    }

    fn scan_number(&mut self, chars: &mut Peekable<Chars<'_>>, source: &'a String) -> Result<(), TError> {
        loop {
            if let Some(next) = chars.peek() {
                match next {
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                        self.advance(chars);
                    },
                    _ => {
                        break;
                    }
                }
            } else {
                break;
            }
        }

        let lexeme = &source[self.start..self.current];
        let number_conversion = lexeme.parse::<i64>();
        if let Ok(number) = number_conversion {
            let literal = Some(LiteralValue::IntValue(number));
            self.add_token(TokenType::IntLiteral, lexeme.to_owned(), literal);
            Ok(())
        } else {
            Err(TError::ScanError(format!("Failed to parse number {:?}", number_conversion)))
        }
    }

}