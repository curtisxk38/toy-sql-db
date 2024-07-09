use crate::{catalog::table_schema::TableSchema, execution::operators::Projection, parse::ast::{CreateTableStatement, Expr}};


pub enum QueryPlan {
    CreateTablePlan(CreateTablePlan),
    InsertPlan(InsertPlan),
    SelectPlan(SelectPlan),
}

pub struct CreateTablePlan {
    pub stmt: CreateTableStatement
}

pub struct InsertPlan {
    pub table: String,
    pub values: Vec<Vec<Expr>>,
}

pub struct SelectPlan {
    pub projection: Projection
}