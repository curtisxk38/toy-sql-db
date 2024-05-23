use crate::{catalog::table_schema::{Column, TableSchema}, parse::ast::{CreateTableStatement, Statement}, planner::query_plan::{CreateTablePlan, QueryPlan}, storage::{buffer_pool::{BufferPoolManager, PageId}, table_page::TablePage}};


pub fn execute(buffer_pool: &mut BufferPoolManager, tables: &mut Vec<TableSchema>, plan: QueryPlan) {
    match plan {
        QueryPlan::CreateTablePlan(plan) => {
            execute_create_table(buffer_pool, tables, &plan)
        },
    }
}

fn execute_create_table(buffer_pool: &mut BufferPoolManager, tables: &mut Vec<TableSchema>, plan: &CreateTablePlan) {
    let stmt = &plan.stmt;
    let table_name = stmt.token.lexeme.clone();
    let columns = stmt.columns.iter().map(|c| {
        Column {name: c.token.lexeme.clone(), column_type: match c.column_type {
            crate::parse::ast::ColumnType::Bool => crate::catalog::table_schema::ColumnType::Bool,
            crate::parse::ast::ColumnType::Int => crate::catalog::table_schema::ColumnType::Int,
        } }
    }).collect();
    

    
    let new_page = buffer_pool.new_page().unwrap();
    let new_page_id = new_page.borrow().get_page_id().unwrap();

    let new_table = TableSchema::new(table_name, columns, new_page_id.0.try_into().unwrap());

    let page = buffer_pool.get_catalog_page().unwrap();
    let mut table = TablePage::new(page);

    table.insert_tuple(new_table.serialize()).unwrap();

    tables.push(new_table);
}