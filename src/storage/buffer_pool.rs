use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::iter;
use std::path::Path;

use crate::config::config::PAGE_SIZE;

use super::disk_manager::DiskManager;
use super::lru_k_replacer::LRUKReplacer;



pub struct PageTableEntry<'a> {
    page: Option<Page<'a>>,
    pin_count: i64,
    is_dirty: bool,
    frame_id: FrameId

}

pub struct Page<'a> {
    page_id: PageId,
    data: &'a mut [u8]
}


impl  <'a> PageTableEntry<'a> {
    pub fn new(frame_id: FrameId) -> PageTableEntry<'a> {
        PageTableEntry {page: None, pin_count: 0, is_dirty: false, frame_id }
    }
}

pub struct DirectoryEntry {

}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct PageId(usize);

impl PageId {
    pub fn temp(&self) -> usize {
        self.0
    }
}

impl From<usize> for PageId {
    fn from(val: usize) -> PageId {
        PageId(val)
    }
}


#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct FrameId(usize);

impl From<usize> for FrameId {
    fn from(val: usize) -> FrameId {
        FrameId(val)
    }
}


pub struct BufferPoolManager<'a> {
    replacer: LRUKReplacer,
    page_table: Vec<PageTableEntry<'a>>,
    page_to_frame: HashMap<PageId, FrameId>,
    memory_pool: Vec<u8>,
    disk_manager: DiskManager,
    // temp
    next_page_id: usize,
}

impl <'a> BufferPoolManager<'a> {
    pub fn new(pool_size: usize, k: usize) -> BufferPoolManager<'a> {
        BufferPoolManager {replacer: LRUKReplacer::new( pool_size, k),
        page_table: (0..pool_size).map(|f| PageTableEntry::new(FrameId::from(f))).collect(),
        page_to_frame: HashMap::new(),
        memory_pool: vec![0u8; pool_size * PAGE_SIZE],
        disk_manager: DiskManager::new(),
            next_page_id: 0,
        }
    }

    pub fn fetch_page (&'a mut self, page_id: PageId) -> &Option<Page<'a>> {
        // return none if no page is available in the free list and all other pages are currently pinned
        match self.page_to_frame.get(&page_id) {
            Some(frame_id) => {
                let pte = &mut self.page_table[frame_id.0];
                self.replacer.record_access(*frame_id);
                // TODO set not evictable?
                let page = Page {
                    page_id, 
                    data: &mut self.memory_pool[(frame_id.0 * PAGE_SIZE) .. (frame_id.0 + 1) * PAGE_SIZE]
                };
                pte.page = Some(page);
                &self.page_table[frame_id.0].page
            },
            None => {
                // TODO use a better datastructure?
                // find first free frame
                
                match self.find_free_frame() {
                    Some(frame_id) => {
                        let buf = self.disk_manager.read_page(&page_id);
                        self.memory_pool[(frame_id.0 * PAGE_SIZE) .. (frame_id.0 + 1) * PAGE_SIZE].copy_from_slice(&buf);

                        let pte = &mut self.page_table[frame_id.0];
                        self.replacer.record_access(frame_id);
                        // TODO set not evictable?
                        let page = Page {
                            page_id: page_id.clone(), 
                            data: &mut self.memory_pool[(frame_id.0 * PAGE_SIZE) .. (frame_id.0 + 1) * PAGE_SIZE]
                        };

                        self.page_to_frame.insert(page_id, frame_id);
                        pte.page = Some(page);
                        return &self.page_table[frame_id.0].page;
                    }
                    None => {
                        
                        // no free frames, have to evict
                        match self.replacer.evict() {
                            Ok(frame_id) => {
                                // remove old frame
                                self.remove_old_page_from_frame(frame_id);
                                // read in new frame
                                
                                let buf = self.disk_manager.read_page(&page_id);
                                self.memory_pool[(frame_id.0 * PAGE_SIZE) .. (frame_id.0 + 1) * PAGE_SIZE].copy_from_slice(&buf);
                                
                                
                                //self.load_page_into_frame(&page_id, frame_id);
                                let pte = &mut self.page_table[frame_id.0];
                                self.replacer.record_access(pte.frame_id);
                                let page = Page {
                                    page_id: page_id.clone(), 
                                    data: &mut self.memory_pool[(pte.frame_id.0 * PAGE_SIZE) .. (pte.frame_id.0 + 1) * PAGE_SIZE]
                                };
                                
                                self.page_to_frame.insert(page_id.clone(), pte.frame_id);
                                pte.page = Some(page);
                                pte.is_dirty = false;

                                
                                return &self.page_table[frame_id.0].page
                            },
                            Err(_) => {
                                &None
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_page_into_frame(&'a mut self, page_id: &PageId, frame_id: FrameId) {
        let pte = &mut self.page_table[frame_id.0];
        self.replacer.record_access(pte.frame_id);
        let page = Page {
            page_id: page_id.clone(), 
            data: &mut self.memory_pool[(pte.frame_id.0 * PAGE_SIZE) .. (pte.frame_id.0 + 1) * PAGE_SIZE]
        };
        
        self.page_to_frame.insert(page_id.clone(), pte.frame_id);
        pte.page = Some(page);
    }

    fn remove_old_page_from_frame(&mut self, frame_id: FrameId) {
        let pte = &mut self.page_table[frame_id.0];
        // remove old frame
        let old_page = pte.page.as_ref().unwrap();
        if pte.is_dirty {
            self.disk_manager.write_page(&old_page.page_id, &self.memory_pool[(frame_id.0 * PAGE_SIZE) .. (frame_id.0 + 1) * PAGE_SIZE]);
        
        }
        self.page_to_frame.remove(&old_page.page_id);
        self.replacer.remove(frame_id);
    }

    fn find_free_frame(&self) -> Option<FrameId> {
        for (frame_id, pte) in self.page_table.iter().enumerate() {
            if pte.page.is_none() {
                return Some(FrameId::from(frame_id));
            }
        }
        None
    }
    
    pub fn unpin_page(&mut self, page_id: PageId, is_dirty: bool) {
        // TODO false if the page is not in the page table or its pin count is <= 0 before this call, true otherwise
        let frame_id = self.page_to_frame.get(&page_id).unwrap();
        self.page_table[frame_id.0].pin_count -= 1;
        self.page_table[frame_id.0].is_dirty |= is_dirty;
        // TODO call replacer.set_evictable
    }
    
    pub fn flush_page(&mut self, page_id: &PageId) {
        // flush a page regardless of its pin status.
        // Flush the target page to disk
        let frame_id = self.page_to_frame.get(page_id).unwrap();
        let pte = &mut self.page_table[frame_id.0];
        
        let page = pte.page.as_ref().unwrap();
        self.disk_manager.write_page(&page.page_id, page.data);

    }
    
    pub fn new_page(&self) -> Option<Page> {
        // create a new page for new data that isnt in any page yet

        None
    }
    
    pub fn delete_page(&mut self, page_id: &PageId) -> bool {
        let frame_id = self.page_to_frame.get(page_id).unwrap().clone();
        let pte = &self.page_table[frame_id.0];
        if pte.pin_count > 0 {
            return false;
        }
        // TODO disk manager delete page?
        self.replacer.remove(frame_id);
        self.page_to_frame.remove(page_id);
        self.flush_page(page_id);
        let pte = &mut self.page_table[frame_id.0];
        pte.page = None;
        pte.pin_count = 0;
        pte.is_dirty = false;
        return true;        
    }

    
    pub fn flush_all_pages(&mut self) {
        let mut page_ids = Vec::new();
        for page_table_entry in &self.page_table {
            if let Some(page) = &page_table_entry.page {
                page_ids.push(page.page_id.clone());
            }
        }
        for page_id in &page_ids {
            self.flush_page(page_id)
        }
        
    }
    
}


