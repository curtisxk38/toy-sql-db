use std::{fmt::format, iter::Peekable, slice::Iter};

use super::{ast::{Column, ColumnType, CreateTableStatement, Expr, InsertStatement, Literal, Statement}, scanner::TError, token::{Token, TokenType}};



pub struct Parser {
    pub errors: Vec<TError>,
}

impl Parser {
    

    pub fn new() -> Parser {
        Parser { errors: Vec::new() }
    }

    // query -> statement* EOF ;
    pub fn parse(&mut self, tokens: & Vec<Token>) -> Result<Vec<Statement>, ()> {
        let mut tokens = tokens.iter().peekable();
        let mut statements: Vec<Statement> = Vec::new();

        loop {
            match tokens.peek() {
                Some(token) => {
                    match token.token_type {
                        TokenType::EOF => {
                            break
                        },
                        _ => {
                            let result = self.statement(&mut tokens);
                            match result {
                                Ok(s) => {
                                    statements.push(s)
                                },
                                Err(e) => {
                                    self.errors.push(e);
                                    self.synchronize(&mut tokens);
                                }
                            }
                        }
                    }
                }
                None => {
                    break;
                }
            }
            
        }

        if self.errors.len() > 0 {
            Err(())
        } else {
            Ok(statements)
        }
    }

    fn synchronize(&mut self, tokens: &mut Peekable<Iter<Token>>) {
        // in case of error, try to get to a normal state to report more errors
        let mut next = tokens.next();

        loop {
            match next {
                Some(token) => {
                    // if we just consumed a semicolon,
                    // we're synchronized and ready to parse the next statement
                    match token.token_type {
                        TokenType::Semicolon => break,
                        _ => {}
                    };
        
                    match tokens.peek() {
                        // if the next token in the list is one of the below
                        // we are ready to start parsing the next statement,
                        // since these token types all are used to start statements
                        Some(peeked) => {
                            match peeked.token_type {
                                TokenType::Select => break,
                                TokenType::Insert => break,
                                TokenType::Create => break,
                                _ => {}
                            }
                        },
                        None => break
                    };
        
                    next = tokens.next();
                },
                None => {
                    break;
                }
            }
        }
    }

    // statement -> select | insert | create_table
    fn statement(&mut self, tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, TError> {
        match &tokens.peek().unwrap().token_type {
            TokenType::Select => {
                self.select(tokens)
            },
            TokenType::Insert => {
                self.insert(tokens)
            },
            TokenType::Create => {
                self.create_table(tokens)
            }
            _ => {
                let token = tokens.peek().unwrap();
                Err(TError::ParseError(
                
                format!("found unexpected token {:?} at line {}", token, token.line)
            ))
            }
        }
    }
    
    fn select(&self, tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, TError> {
        todo!()
    }
    
    // insert -> "insert" "into" identifier "(" identifier ("," identifier)*  ")" "values" value+ ";"
    fn insert(&self, tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, TError> {
        tokens.next(); // consume "insert"

        match tokens.peek().unwrap().token_type {
            TokenType::Into => {
                tokens.next(); // consume "into"
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected 'into' after insert", token, token.line)
                ))
            }
        };

        let identifier;
        match tokens.peek().unwrap().token_type {
            TokenType::Identifier => {
                identifier = tokens.next().unwrap(); // consume identifier
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected identifier", token, token.line)
                ))
            }
        };

        match tokens.peek().unwrap().token_type {
            TokenType::LeftParen => {
                tokens.next(); // consume "("
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected '('", token, token.line)
                ))
            }
        };

        let mut columns = Vec::new();
        loop {
            match tokens.peek().unwrap().token_type {
                TokenType::Identifier => {
                    let identifier = tokens.next().unwrap(); // consume identifier
                    columns.push(identifier.lexeme.clone());
                    match tokens.peek().unwrap().token_type {
                        TokenType::Comma => {
                            tokens.next(); //consume ","
                        }
                        TokenType::RightParen => {
                            tokens.next(); //consume ")"
                            break;
                        },
                        _ => {

                        }
                    }
                },
                
                _ => {
                    let token = tokens.peek().unwrap();
                    return Err(TError::ParseError(
                        format!("found unexpected {:?} at line {}. expected identifier", token, token.line)
                    ))
                }
            };
        };

        match tokens.peek().unwrap().token_type {
            TokenType::Values => {
                tokens.next(); // consume "values"
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected 'values'", token, token.line)
                ))
            }
        };

        let values = self.values(tokens)?;

        match tokens.peek().unwrap().token_type {
            TokenType::Semicolon => {
                tokens.next(); // consume ";"
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected ';' after values", token, token.line)
                ))
            }
        };

        Ok(Statement::InsertStatement(InsertStatement { token: identifier.clone(), columns, values}))

    }
    
    // values -> ( "(" expr ("," expr)* ")" )+
    fn values(&self, tokens: &mut Peekable<Iter<Token>>) -> Result<Vec<Vec<Expr>>, TError> {
        let mut res = Vec::new();
        loop {
            match tokens.peek().unwrap().token_type {
                TokenType::LeftParen => {
                    tokens.next(); // consume "("
                }
                _ => {
                    let token = tokens.peek().unwrap();
                    return Err(TError::ParseError(
                        format!("found unexpected {:?} at line {}. expected identifier", token, token.line)
                    ))
                }
            };
            let mut value = Vec::new();
            loop {
                let expr = self.expr(tokens)?;
                value.push(expr);
                match tokens.peek().unwrap().token_type {
            
                        TokenType::Comma => {
                            tokens.next(); //consume ","
                        }
                        TokenType::RightParen => {
                            tokens.next(); //consume ")"
                            break;
                        },
                        _ => {

                        }
                    }
            };
            res.push(value);
            match tokens.peek().unwrap().token_type {
                TokenType::Comma => {
                    tokens.next(); // consume ","
                }
                _ => {
                    break;
                }
            };
        };
        Ok(res)
    }

    // expr -> literal | column_reference
    fn expr(&self, tokens: &mut Peekable<Iter<Token>>) -> Result<Expr, TError> {
        match tokens.peek().unwrap().token_type {
            TokenType::IntLiteral | TokenType::False | TokenType::True | TokenType::Null => {
                let token = tokens.next().unwrap();
                Ok(Expr::Literal(Literal { token: token.clone(), value: token.literal.as_ref().unwrap().clone() }))
            }
            TokenType::Identifier => {
                todo!()
            },
            _ => {
                let token = tokens.peek().unwrap();
                Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected literal", token, token.line)
                ))
            }
        }
    }
    
    // create_table -> "CREATE" "TABLE" identifier "(" Column+ ")" ";"
    fn create_table(&self, tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, TError> {
        tokens.next(); // consume "create"

        match tokens.peek().unwrap().token_type {
            TokenType::Table => {
                tokens.next(); // consume "table"
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected 'table' after create", token, token.line)
                ))
            }
        };

        let identifier;
        match tokens.peek().unwrap().token_type {
            TokenType::Identifier => {
                identifier = tokens.next().unwrap(); // consume identifier
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected identifier", token, token.line)
                ))
            }
        };

        match tokens.peek().unwrap().token_type {
            TokenType::LeftParen => {
                tokens.next(); // consume "("
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected '('", token, token.line)
                ))
            }
        };

        let mut columns: Vec<Column> = Vec::new();

        let first_column: Column = self.column(tokens)?;
        columns.push(first_column);

        loop {
            

            match tokens.peek().unwrap().token_type {
                TokenType::RightParen => {
                    tokens.next().unwrap(); // consume ")"
                    break;
                },
                _ => {}
            };

            match tokens.peek().unwrap().token_type {
                TokenType::Comma => {
                    tokens.next().unwrap(); // consume ","
                },
                _ => {
                    let token = tokens.peek().unwrap();
                    return Err(TError::ParseError(
                        format!("found unexpected {:?} at line {}. expected ','", token, token.line)
                    ))
                }
            };

            let column: Column = self.column(tokens)?;
            columns.push(column);

            
        }
        match tokens.peek().unwrap().token_type {
            TokenType::Semicolon => {
                tokens.next(); // consume ";"
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected ';'", token, token.line)
                ))
            }
        };

        Ok(Statement::CreateTableStatement(CreateTableStatement {token: identifier.clone(), columns}))
    }

    // column -> identifier ("int" | "bool")
    fn column(&self, tokens: &mut Peekable<Iter<Token>>) -> Result<Column, TError> {
        let identifier;
        match tokens.peek().unwrap().token_type {
            TokenType::Identifier => {
                identifier = tokens.next().unwrap(); // consume identifier
            },
            _ => {
                let token = tokens.peek().unwrap();
                return Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected identifier", token, token.line)
                ))
            }
        };

        let column_t = self.column_type(tokens)?;
        Ok(Column {token: identifier.clone(), column_type: column_t})
        
    }

    fn column_type(&self, tokens: &mut Peekable<Iter<Token>>) -> Result<ColumnType, TError> {
        match tokens.peek().unwrap().token_type {
            TokenType::Int => {
                tokens.next().unwrap(); // consume "int"
                Ok(ColumnType::Int)
            },
            TokenType::Bool => {
                tokens.next().unwrap(); // consume "bool"
                Ok(ColumnType::Int)
            }
            _ => {
                let token = tokens.peek().unwrap();
                Err(TError::ParseError(
                    format!("found unexpected {:?} at line {}. expected valid column type", token, token.line)
                ))
            }
        }
    }
}

