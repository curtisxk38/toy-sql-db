use std::{fs::{self, File}, io, path::PathBuf};

use config::config::PAGE_SIZE;
use parse::{parser::Parser, scanner::Scanner};
use storage::buffer_pool::BufferPoolManager;

mod storage;
mod config;
mod parse;
mod catalog;

fn init() -> std::io::Result<()> {
    println!("init");
    
    let _ = fs::create_dir(config::config::DATA_DIR);
    let dir = PathBuf::from(config::config::DATA_DIR);
    let data_file_path = dir.join(config::config::DATA_FILE);
    if !data_file_path.exists() {
        File::create(config::config::DATA_FILE).unwrap();
    }
    Ok(())
}

fn cleanup() -> std::io::Result<()> {
    fs::remove_dir_all(config::config::DATA_DIR)?;
    println!("cleaned up!");
    Ok(())
}


fn main() {
    init();
    let mut input = String::new();
    let stdin = io::stdin();
    let pool_size=4;
    let mut memory = vec![0u8; pool_size * PAGE_SIZE];
    let buffer_pool = BufferPoolManager::new(&mut memory, pool_size, 2);
    let mut scanner = Scanner::new();
    let mut parser = Parser::new();
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
                                println!("{:?}", statements)
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
    //storage::buffer_pool::get_page(3);
    let _ = cleanup();
    
}
