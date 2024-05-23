use crate::parse::ast::CreateTableStatement;


pub enum QueryPlan {
    CreateTablePlan(CreateTablePlan)
}

pub struct CreateTablePlan {
    pub stmt: CreateTableStatement
}