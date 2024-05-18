use crate::storage::{buffer_pool::{self, BufferPoolManager, PageId}, table_page::{TablePage, TupleId}};

use super::table_schema::{self, TableSchema};



pub fn load_catalog(buffer_pool: &mut BufferPoolManager) -> Vec<TableSchema> {
    let mut tables = Vec::new();
    let p = buffer_pool.get_catalog_page();
    match p {
        Some(p) => {
            let table = TablePage::new(p);

            // TODO look through next page
            for tuple_id in 0..table.get_num_tuples() {
                let tuple_id = TupleId(tuple_id.into());
                let table_schema_tuple = table.get_tuple(tuple_id);
                let table_schema = TableSchema::deserialize(table_schema_tuple);
                tables.push(table_schema);
            }
            
        },
        None => {}
    }
    tables
}