use std::{fs::{self, File, OpenOptions}, io, path::PathBuf, vec};

use catalog::{table_schema::TableSchema};
use config::config::PAGE_SIZE;
use parse::{parser::Parser, scanner::Scanner};
use storage::buffer_pool::BufferPoolManager;

use crate::{catalog::catalog::load_catalog, execution::execution::execute, planner::planner::plan};

mod storage;
mod config;
mod parse;
mod catalog;
mod planner;
mod execution;



fn init(buffer_pool: &mut BufferPoolManager) -> std::io::Result<Vec<TableSchema>> {
    println!("init");
    
    let dir = PathBuf::from(config::config::DATA_DIR);
    let data_file_path = dir.join(config::config::DATA_FILE);
    let tables;
    if !data_file_path.exists() {
        File::create(config::config::DATA_FILE).unwrap();
        tables = vec![];
    } else {
        let file = OpenOptions::new().read(true).open(data_file_path).unwrap();
        if file.metadata().unwrap().len() > 0 {
            tables = load_catalog(buffer_pool);
        } else {
            tables = Vec::new();
        }
        
    }
    Ok(tables)
}

fn cleanup(buffer_pool: &mut BufferPoolManager) {
    buffer_pool.flush_all_pages();
    println!("cleaned up!");
}


fn main() {
    
    let pool_size=4;
    let mut memory = vec![0u8; pool_size * PAGE_SIZE];
    let mut buffer_pool = BufferPoolManager::new(&mut memory, pool_size, 2);
    let mut scanner = Scanner::new();
    let mut parser = Parser::new();

    let mut tables = init(&mut buffer_pool).unwrap();

    let mut input = String::new();
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::Write::flush(&mut io::stdout()).ok().expect("Couldn't flush stdout");
        input.clear();
        let read = stdin.read_line(&mut input);
        match read {
            Ok(chars_read) => { 
                if chars_read == 0 {
                    break;
                }
                match scanner.scan(&input) {
                    Ok(_) => {
                        let r = parser.parse(&scanner.tokens);
                        match r {
                            Ok(statements) => {
                                println!("{:?}", statements);
                                for stmt in statements {
                                    let plan = plan(&mut tables, stmt).expect("plan");
                                    execute(&mut buffer_pool, &mut tables, plan);
                                }
                            },
                            Err(_) => {
                                println!("tokens: {:?}", scanner.tokens);
                                for error in &parser.errors {
                                    println!("{:?}", error);
                                }
                                parser.errors.clear();
                            },
                        }
                    },
                    Err(e) => {
                        println!("error: {:?}", e);
                    },
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }

    cleanup(&mut buffer_pool);
    
}
