use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use super::lru_k_replacer::LRUKReplacer;


pub const DATA_DIR: &str = "data";

#[derive(Clone)]
pub struct PageTableEntry {
    page_id: Option<PageId>,
    pin_count: i64,
    dirty: bool,

}

pub struct Page {

}


impl PageTableEntry {
    pub fn new() -> PageTableEntry {
        PageTableEntry {page_id: None, pin_count: 0, dirty: false }
    }
}

pub struct DirectoryEntry {

}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct PageId(usize);

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct FrameId(usize);

impl From<usize> for FrameId {
    fn from(val: usize) -> FrameId {
        FrameId(val)
    }
}


pub struct BufferPoolManager {
    replacer: LRUKReplacer,
    page_table: Vec<PageTableEntry>,
    page_to_frame: HashMap<PageId, FrameId>
}

impl BufferPoolManager {
    pub fn new(pool_size: usize, k: usize) -> BufferPoolManager {
        BufferPoolManager {replacer: LRUKReplacer::new( pool_size, k),
        page_table: vec![PageTableEntry::new(); pool_size],
        page_to_frame: HashMap::new(),
    
        }
    }

    pub fn fetch_page(&self, page_id: PageId) -> Option<Page> {
        // return none if no page is available in the free list and all other pages are currently pinned
        match self.page_to_frame.get(&page_id) {
            Some(frame_id) => {
                let page = self.page_table[frame_id.0].dirty;
                todo!()
            },
            None => {
                // TODO use a better datastructure?
                for pte in &self.page_table {
                    if pte.page_id.is_none() {
                        todo!()
                    }
                }
                // no free frames, have to evict
                match self.replacer.evict() {
                    Ok(frame_id) => {
                        let page = self.page_table[frame_id.0].dirty;
                        todo!()
                    },
                    Err(_) => {
                        None
                    }
                }
            }
        }
    }
    
    pub fn unpin_page(&mut self, page_id: PageId, is_dirty: bool) {
        let frame_id = self.page_to_frame.get(&page_id).unwrap();
        self.page_table[frame_id.0].pin_count -= 1;
        self.page_table[frame_id.0].dirty |= is_dirty;
    }
    
    pub fn flush_page(&mut self, page_id: &PageId) {
        // flush a page regardless of its pin status.
        let frame_id = self.page_to_frame.get(page_id).unwrap();
    }
    
    pub fn new_page(&self, page_id: PageId) {
    }
    
    pub fn delete_page(&self) {
    
    }
    
    pub fn flush_all_pages(&mut self) {
        let mut page_ids = Vec::new();
        for page_table_entry in &self.page_table {
            if let Some(page_id) = &page_table_entry.page_id {
                page_ids.push(page_id.clone());
            }
        }
        for page_id in &page_ids {
            self.flush_page(page_id)
        }
        
    }
    
}


