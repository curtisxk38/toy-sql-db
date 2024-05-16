use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::iter;
use std::path::Path;
use std::rc::Rc;

use crate::config::config::PAGE_SIZE;

use super::disk_manager::DiskManager;
use super::lru_k_replacer::LRUKReplacer;



pub struct PageTableEntry<'a> {
    page_id: Option<PageId>,
    pin_count: i64,
    is_dirty: bool,
    frame_id: FrameId,
    data: Rc<RefCell<&'a mut [u8]>>,

}


impl  <'a> PageTableEntry<'a> {
    pub fn new(frame_id: FrameId, data: &'a mut [u8]) -> PageTableEntry<'a> {
        PageTableEntry {page_id: None, pin_count: 0, is_dirty: false, frame_id, data: Rc::from(RefCell::from(data)) }
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
    disk_manager: DiskManager,
    // temp
    next_page_id: usize,
}

impl <'a> BufferPoolManager<'a> {
    pub fn new(memory: &'a mut Vec<u8>,pool_size: usize, k: usize) -> BufferPoolManager<'a> {
        
        BufferPoolManager {replacer: LRUKReplacer::new( pool_size, k),
        page_table: memory.chunks_exact_mut(PAGE_SIZE).enumerate().map(|(index, memory)| 
        PageTableEntry::new(FrameId::from(index), memory)
    ).collect(),
        page_to_frame: HashMap::new(),
        disk_manager: DiskManager::new(),
        next_page_id: 0,
        }
    }

    pub fn fetch_page (&mut self, page_id: PageId) -> Option<&PageTableEntry<'a>> {
        // return none if no page is available in the free list and all other pages are currently pinned
        match self.page_to_frame.get(&page_id) {
            Some(frame_id) => {
                let pte = &mut self.page_table[frame_id.0];
                self.replacer.record_access(*frame_id);
                // TODO set not evictable?
                pte.page_id = Some(page_id);
                Some(&self.page_table[frame_id.0])
            },
            None => {
                // TODO use a better datastructure?
                // find first free frame
                
                match self.find_free_frame() {
                    Some(frame_id) => {
                        let pte = &mut self.page_table[frame_id.0];
                        
                        let buf = self.disk_manager.read_page(&page_id);
                        pte.data.borrow_mut().copy_from_slice(&buf);

                        
                        self.replacer.record_access(frame_id);
                        // TODO set not evictable?

                        self.page_to_frame.insert(page_id.clone(), frame_id);
                        pte.page_id = Some(page_id);
                        return Some(&self.page_table[frame_id.0]);
                    }
                    None => {
                        
                        // no free frames, have to evict
                        match self.replacer.evict() {
                            Ok(frame_id) => {
                                // remove old frame
                                
                                self.remove_old_page_from_frame(&frame_id);
                                // read in new frame
                                let pte = &mut self.page_table[frame_id.0];
                                let buf = self.disk_manager.read_page(&page_id);
                                pte.data.borrow_mut().copy_from_slice(&buf);
                                
                                self.replacer.record_access(pte.frame_id);
                                
                                self.page_to_frame.insert(page_id.clone(), pte.frame_id);
                                pte.page_id = Some(page_id);
                                pte.is_dirty = false;

                                
                                return Some(&self.page_table[frame_id.0]);
                            },
                            Err(_) => {
                                None
                            }
                        }
                    }
                }
            }
        }
    }

    fn remove_old_page_from_frame(&mut self, frame_id: &FrameId) {
        let pte = &mut self.page_table[frame_id.0];
        let page_id = pte.page_id.as_ref().unwrap().clone();
        // remove old frame
        if pte.is_dirty {
            self.disk_manager.write_page(&page_id, &pte.data.borrow());
        
        }
        self.page_to_frame.remove(&page_id);
        self.replacer.remove(pte.frame_id);
    }

    fn find_free_frame(&self) -> Option<FrameId> {
        for pte in self.page_table.iter() {
            if pte.page_id.is_none() {
                return Some(pte.frame_id);
            }
        }
        None
    }
    
    pub fn unpin_page(&mut self, page_id: PageId, is_dirty: bool) {
        // TODO false if the page is not in the page table or its pin count is <= 0 before this call, true otherwise
        let frame_id = self.page_to_frame.get(&page_id).unwrap();
        self.page_table[frame_id.0].pin_count -= 1;
        self.page_table[frame_id.0].is_dirty |= is_dirty;
        if self.page_table[frame_id.0].pin_count <= 0 {
            self.replacer.set_evictable(frame_id.clone(), true)
        }
        
    }
    
    pub fn flush_page(&mut self, page_id: &PageId) {
        // flush a page regardless of its pin status.
        // Flush the target page to disk
        let frame_id = self.page_to_frame.get(page_id).unwrap();
        let pte = &mut self.page_table[frame_id.0];
        
        self.disk_manager.write_page(pte.page_id.as_ref().unwrap(), &pte.data.borrow());

    }
    
    pub fn new_page(&mut self) -> Option<&PageTableEntry<'a>> {
        // create a new page for new data that isnt in any page yet
        
        match self.find_free_frame() {
            Some(frame_id) => {
                let pte = &mut self.page_table[frame_id.0];
                let page_id = PageId::from(self.next_page_id);
                self.next_page_id += 1;
                // zero out data
                pte.data.borrow_mut().fill(0);

                
                self.replacer.record_access(frame_id);
                // TODO set not evictable?

                self.page_to_frame.insert(page_id.clone(), frame_id);
                pte.page_id = Some(page_id);
                pte.is_dirty = false;
                return Some(&self.page_table[frame_id.0]);
            }
            None => {
                // no free frames, have to evict
                match self.replacer.evict() {
                    Ok(frame_id) => {
                        // remove old frame
                        self.remove_old_page_from_frame(&frame_id);

                        let pte = &mut self.page_table[frame_id.0];
                        let page_id = PageId::from(self.next_page_id);
                        self.next_page_id += 1;
                        // zero out data
                        pte.data.borrow_mut().fill(0);
                    
                        
                        self.replacer.record_access(pte.frame_id);
                        
                        self.page_to_frame.insert(page_id.clone(), pte.frame_id);
                        pte.page_id = Some(page_id);
                        pte.is_dirty = false;

                        
                        return Some(&self.page_table[frame_id.0]);
                    },
                    Err(_) => {
                        None
                    }
                }
            }
        }
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
        pte.page_id = None;
        pte.pin_count = 0;
        pte.is_dirty = false;
        return true;        
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


#[cfg(test)]
mod tests {
    use crate::{config::config::PAGE_SIZE, storage::buffer_pool::{BufferPoolManager, FrameId, PageId}};

    #[test]
    fn simple() {
        let pool_size= 4;
        let mut memory = vec![0u8; pool_size * PAGE_SIZE];
        let mut buffer_pool = BufferPoolManager::new(&mut memory, pool_size, 2);
        let p = buffer_pool.new_page().unwrap();
        assert!(p.page_id == Some(PageId(0)));
        let pte = buffer_pool.fetch_page(PageId::from(0)).unwrap();
        assert_eq!(vec![0; PAGE_SIZE], *pte.data.borrow());
    }

    #[test]
    fn simple2() {
        let pool_size= 2;
        let mut memory = vec![0u8; pool_size * PAGE_SIZE];
        let mut buffer_pool = BufferPoolManager::new(&mut memory, pool_size, 2);
        let p = buffer_pool.new_page().unwrap();
        assert!(p.page_id == Some(PageId(0)));
        let p2 = buffer_pool.new_page().unwrap();
        assert!(p2.page_id == Some(PageId(1)));
        buffer_pool.unpin_page(PageId(0), false);
        let p3 = buffer_pool.new_page().unwrap();
        assert!(p3.page_id == Some(PageId(2)));
    }
}
