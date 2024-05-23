use crate::parse::ast::{CreateTableStatement, Expr};


pub enum QueryPlan {
    CreateTablePlan(CreateTablePlan),
    InsertPlan(InsertPlan),
}

pub struct CreateTablePlan {
    pub stmt: CreateTableStatement
}

pub struct InsertPlan {
    pub table: String,
    pub values: Vec<Vec<Expr>>,
}