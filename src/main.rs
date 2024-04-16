use std::{fs, io};

mod storage;

fn init() -> std::io::Result<()> {
    println!("init");
    let _ = fs::create_dir(storage::buffer_pool::DATA_DIR);
    let _ = storage::buffer_pool::create_directory_page();
    Ok(())
}

fn cleanup() -> std::io::Result<()> {
    fs::remove_dir_all(storage::buffer_pool::DATA_DIR)?;
    println!("cleaned up!");
    Ok(())
}


fn main() {
    init();
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
                //self.run(&input);
            }
            Err(error) => println!("error: {}", error),
        }
    }
    //storage::buffer_pool::get_page(3);
    let _ = cleanup();
    
}
