use std::alloc::{alloc_zeroed, dealloc, Layout};

const PAGE_SIZE: i64 = 4096; // 4 KB

struct DiskManager {
    memory: *mut u8,
    length: usize
}

impl DiskManager {
    pub fn new(pages: i64) -> DiskManager {
            let length = PAGE_SIZE * pages;
            let length: usize =  length.try_into().unwrap();
            let layout = Layout::from_size_align(length, std::mem::size_of::<u8>()).expect("failed layout");
            unsafe {
            let ptr = alloc_zeroed(layout);
            DiskManager {memory: ptr, length}
            }
    }
}

impl Drop for DiskManager {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.length, std::mem::size_of::<u8>()).expect("failed layout");
        unsafe {
            dealloc(
                self.memory,
                layout
            )
        };
    }
}
