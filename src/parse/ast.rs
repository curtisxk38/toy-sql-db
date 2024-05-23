use super::token::{LiteralValue, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    ColumnReference(ColumnReference),
    Literal(Literal),
}
#[derive(Debug, Clone)]
pub struct Literal {
    pub token: Token,
    pub value: LiteralValue,
}
#[derive(Debug, Clone)]
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
    pub token: Token,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expr>>, // todo fix this probably
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