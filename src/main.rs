use std::{fs::{self, File}, io, path::PathBuf};

use storage::buffer_pool::BufferPoolManager;

mod storage;
mod config;


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
    let buffer_pool = BufferPoolManager::new(4, 2);
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
                //self.run(&input);
            }
            Err(error) => println!("error: {}", error),
        }
    }
    //storage::buffer_pool::get_page(3);
    let _ = cleanup();
    
}
