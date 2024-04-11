use std::fs;

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
    //storage::buffer_pool::get_page(3);
    //cleanup();
    
}
