

use crate::config::config::{self, DATA_FILE, PAGE_SIZE};

use super::buffer_pool::{FrameId, PageId};
use std::{fs::{self, File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, path::PathBuf};

pub struct DiskManager {
    file_dir: PathBuf,
    file: File,
}

struct PathIndex(pub usize);





struct PageLocation {
    page_id: PageId,
    file: PathBuf,
    index: PathIndex,
}

impl DiskManager {
    pub fn new() -> DiskManager {
            fs::create_dir_all(config::DATA_DIR).unwrap();
            let dir = PathBuf::from(config::DATA_DIR);
            let path = dir.join(config::DATA_FILE);
            let file = OpenOptions::new().write(true).read(true).create(true).open(path).unwrap();
            DiskManager {file_dir: dir, file
            }
    }

    fn get_file(&self, page_id: &PageId) -> PageLocation{
        // for now all pages live in one file
        let index: usize = page_id.0;
        PageLocation {page_id: page_id.clone(), file: self.file_dir.join(DATA_FILE), index: PathIndex(index) }
    }

    pub fn write_page(&mut self, page_id: &PageId, data: &[u8]) {
        let loc = self.get_file(page_id);
        self.file.seek(SeekFrom::Start((loc.index.0 * PAGE_SIZE).try_into().unwrap())).unwrap();
        self.file.write(data).unwrap();
        self.file.flush().unwrap();
    }

    pub fn read_page(&mut self, page_id: &PageId) -> Vec<u8> {
        // for now all pages rae in one file
        let loc = self.get_file(page_id);
        self.file.seek(SeekFrom::Start((loc.index.0 * PAGE_SIZE).try_into().unwrap())).unwrap();
        let mut buffer = [0; PAGE_SIZE]; // TODO take mutable slice as param and .read into it directly
        self.file.read(&mut buffer).unwrap();
        buffer.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::config::PAGE_SIZE, storage::buffer_pool::{FrameId, PageId}};

    use super::DiskManager;


    #[test]
    fn simple() {
        let mut dm = DiskManager::new();
        let data = vec![1; PAGE_SIZE];
        let p = PageId::from(0);
        dm.write_page(&p, &data);
        let r = dm.read_page(&p);
        assert_eq!(r, data);
    }
}