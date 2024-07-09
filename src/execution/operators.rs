use crate::{catalog::table_schema::TableSchema, parse::{ast::Expr, token::LiteralValue}};


pub trait Operator {
    fn next(&self) -> Option<Vec<LiteralValue>>;
}



pub struct Projection {
    pub expressions: Vec<Expr>,
    pub child: Box<dyn Operator>,
}

pub struct SeqScan {
    pub table: TableSchema
}

impl Operator for SeqScan {
    fn next(&self) -> Option<Vec<LiteralValue> >{
        todo!()
    }
}

impl Operator for Projection {
    fn next(&self) -> Option<Vec<LiteralValue> >{
        todo!()
    }
}