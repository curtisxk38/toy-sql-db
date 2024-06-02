use std::{fs, path::{Path, PathBuf}};

use crate::config;


pub struct TestSetup;

impl Drop for TestSetup {
    fn drop(&mut self) {
        // destroy db
        let dir = PathBuf::from(config::config::DATA_DIR);
        let data_file_path = dir.join(config::config::DATA_FILE);
        if data_file_path.exists() {
            fs::remove_file(data_file_path).unwrap();
        }
    }
}