use crate::{catalog::table_schema::TableSchema, parse::{ast::Statement, scanner::TError}, storage::buffer_pool::BufferPoolManager};

use super::query_plan::{CreateTablePlan, QueryPlan};




pub fn plan(buffer_pool: &mut BufferPoolManager, tables: &mut Vec<TableSchema>, statement: Statement) -> Result<QueryPlan, TError> {
    match statement {
        Statement::SelectStatement(_) => todo!(),
        Statement::InsertStatement(_) => todo!(),
        Statement::CreateTableStatement(stmt) => {
            Ok(QueryPlan::CreateTablePlan(CreateTablePlan {stmt} ))
        }
    }
}