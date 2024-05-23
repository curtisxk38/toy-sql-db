use crate::{catalog::table_schema::{Column, TableSchema}, parse::ast::{CreateTableStatement, Expr, Statement}, planner::query_plan::{CreateTablePlan, InsertPlan, QueryPlan}, storage::{buffer_pool::{BufferPoolManager, PageId}, table_page::TablePage}};


pub fn execute(buffer_pool: &mut BufferPoolManager, tables: &mut Vec<TableSchema>, plan: QueryPlan) {
    match plan {
        QueryPlan::CreateTablePlan(plan) => {
            execute_create_table(buffer_pool, tables, &plan)
        },
        QueryPlan::InsertPlan(plan) => {
            execute_insert_table(buffer_pool, tables, &plan)
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

fn execute_insert_table(buffer_pool: &mut BufferPoolManager, tables: &mut Vec<TableSchema>, plan: &InsertPlan) {
    let schema = tables.iter().find(|x| x.name == plan.table).unwrap();
    let latest_page = find_latest_page(schema);
    let page = buffer_pool.fetch_page(PageId(latest_page.try_into().unwrap())).unwrap();
    let mut table = TablePage::new(page);
    for row in &plan.values {
        let tuple = values_row_to_tuple(row);
        match table.insert_tuple(tuple) {
            Some(_) => {},
            None => {
                panic!("allocate new page for table")
            },
        }
    }
}

fn values_row_to_tuple(values: &Vec<Expr>) -> Vec<u8> {
    let mut res = Vec::new();
    for value in values {
        match value {
            Expr::ColumnReference(_) => todo!(),
            Expr::Literal(l) => match l.value {
                crate::parse::token::LiteralValue::IntValue(i) => {
                    res.extend(i.to_le_bytes());
                },
                crate::parse::token::LiteralValue::StringValue(_) => todo!(),
                crate::parse::token::LiteralValue::BooleanValue(b) => {
                    res.push(match b {
                        true => 1,
                        false => 0,
                    })
                },
                crate::parse::token::LiteralValue::NullValue => todo!(),
            }
        }
    }
    res
}

fn find_latest_page(table: &TableSchema) -> u32 {
    // this is incredibly dumb
    // TODO follow the linked list of pages
    table.first_page_id // for now we assume tables are only ever 1 page big
}