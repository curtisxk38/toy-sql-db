use std::collections::HashMap;

use crate::{catalog::table_schema::{self, ColumnType, TableSchema}, execution::operators::{Projection, SeqScan}, parse::{ast::{Expr, InsertStatement, SelectStatement, Statement}, scanner::TError}, storage::buffer_pool::BufferPoolManager};

use super::query_plan::{CreateTablePlan, InsertPlan, QueryPlan, SelectPlan};




pub fn plan(tables: &mut Vec<TableSchema>, statement: Statement) -> Result<QueryPlan, TError> {
    match statement {
        Statement::SelectStatement(stmt) => {
            plan_select(tables, stmt)
        }
        Statement::InsertStatement(stmt) => {
            plan_insert(tables, stmt)
        },
        Statement::CreateTableStatement(stmt) => {
            Ok(QueryPlan::CreateTablePlan(CreateTablePlan {stmt} ))
        }
    }
}

fn plan_select(tables: &mut Vec<TableSchema>, stmt: SelectStatement) -> Result<QueryPlan, TError> {
    let table_name = stmt.from_item.token.lexeme;
    let mut table_schema = None;
    for table in tables {
        if table_name == table.name {
            
            table_schema = Some(table);
            break;
        }
    };
    if table_schema.is_none() {
        return Err(TError::PlanError(format!("table {:?} not found", table_name)));
    };
    let table_schema = table_schema.unwrap();

    for expr in &stmt.expressions {
        match expr {
            Expr::ColumnReference(col) => {
                let mut found = false;
                for col_def in &table_schema.columns {
                    if col.name == col_def.name {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(TError::PlanError(format!("column {:?} not found", col.name)));
                }
            }
            Expr::Literal(_) => {},
        }
    };
    // for now always do a sequential scan
    let scan = SeqScan { table: table_schema.clone() };
    
    let projection = Projection { expressions: stmt.expressions.clone(), child: Box::new(scan)};
    Ok(QueryPlan::SelectPlan(SelectPlan {projection}))
}

fn plan_insert(tables: &mut Vec<TableSchema>, stmt: InsertStatement) -> Result<QueryPlan, TError> {
    let table_name = stmt.token.lexeme;
    let mut table_schema = None;
    for table in tables {
        if table_name == table.name {
            
            table_schema = Some(table);
            break;
        }
    };
    if table_schema.is_none() {
        return Err(TError::PlanError(format!("table {:?} not found", table_name)));
    };

    let table_schema = table_schema.unwrap();

    // check columns
    let name_to_type = table_schema.columns.iter().map(|c| (c.name.clone(), c.column_type.clone())).collect::<HashMap<String, ColumnType>>();
    
    for c in &stmt.columns {
        if name_to_type.get(c).is_none() {
            return Err(TError::PlanError(format!("no column called {:?} found for table {:?}", c, table_name)));
        }
    }

    let mut ordered_values: Vec<Vec<Expr>> = Vec::new();

    let num_cols = stmt.columns.len();
    // this is kinda dumb
    for row in &stmt.values {
        if row.len() != num_cols {
            return Err(TError::PlanError(format!("expected {:?} values for row {:?}", num_cols, table_name)));
        }
        let mut col_to_expr = HashMap::new();
        for (value_col, col_name) in row.iter().zip(stmt.columns.iter()) {
            let expected_type = name_to_type.get(col_name).unwrap();
            let expr_type = &type_of(value_col);
            if std::mem::discriminant(expected_type) != std::mem::discriminant(expr_type) {
                return Err(TError::PlanError(format!("expected a {:?} got a {:?}", expected_type, expr_type)));
            }
            col_to_expr.insert(col_name, value_col);
        }

        let ordered_value = table_schema.columns.iter().map(|c| (*(*col_to_expr.get(&c.name).unwrap())).clone() ).collect::<Vec<Expr>>();
        ordered_values.push(
            ordered_value
        );
        
    }
    Ok(QueryPlan::InsertPlan(InsertPlan {table: table_name, values: ordered_values}))

}

fn type_of(expr: &Expr) -> ColumnType {
    match expr {
        Expr::ColumnReference(_) => todo!(),
        Expr::Literal(lit) => match lit.value {
            crate::parse::token::LiteralValue::IntValue(_) => ColumnType::Int,
            crate::parse::token::LiteralValue::StringValue(_) => todo!(),
            crate::parse::token::LiteralValue::BooleanValue(_) => ColumnType::Bool,
            crate::parse::token::LiteralValue::NullValue => todo!(),
        }
    }
}