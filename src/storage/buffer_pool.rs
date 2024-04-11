use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


pub const DATA_DIR: &str = "data";

pub struct Page {
    page_id: Option<i64>,
    pin_count: i64,
    dirty: bool,

}

impl Page {
    pub fn new() -> Page {
        Page {page_id: None, pin_count: 0, dirty: false }
    }
}

pub struct DirectoryEntry {

}

pub fn fetch_page(page_id: i64) -> Option<Page> {
    // return none if no page is available in the free list and all other pages are currently pinned
    None
}

pub fn unpin_page(page_id: i64, is_dirty: bool) {

}

pub fn flush_page(page_id: i64) {
    // flush a page regardless of its pin status.
}

pub fn new_page() {
}

pub fn delete_page() {

}

pub fn flush_all_pages() {
    
}

pub fn create_directory_page() -> std::io::Result<()>{
    let mut file = File::create(Path::new(DATA_DIR).join(Path::new("directory")))?;
    file.write_all(b"Hello, world!")?;
    Ok(())
}

pub fn write_page(p: Page) {

}

