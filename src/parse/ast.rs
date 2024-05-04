use super::token::{LiteralValue, Token};

#[derive(Debug)]
pub enum Expr {
    ColumnReference(ColumnReference),
    Literal(Literal),
}
#[derive(Debug)]
pub struct Literal {
    pub token: Token,
    pub value: LiteralValue,
}
#[derive(Debug)]
pub struct ColumnReference {

}
#[derive(Debug)]
pub enum Statement {
    SelectStatement(SelectStatement),
    InsertStatement(InsertStatement),
    CreateTableStatement(CreateTableStatement),
}
#[derive(Debug)]
pub struct SelectStatement {
    pub expressions: Vec<Expr>,
    pub from_item: Table,
}

#[derive(Debug)]
pub struct Table {
    token: Token,
}

#[derive(Debug)]
pub struct InsertStatement {
    
}
#[derive(Debug)]
pub struct CreateTableStatement {
    pub token: Token,
    pub columns: Vec<Column>,
}
#[derive(Debug)]
pub struct Column {
    pub token: Token,
    pub column_type: ColumnType
}
#[derive(Debug)]
pub enum ColumnType {
    Bool,
    Int
}