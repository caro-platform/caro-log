use std::{collections::VecDeque, os::unix::prelude::MetadataExt, path::PathBuf};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShiftDirection {
    Left,
    Right,
}

pub trait LogFile {
    fn shift_and_read(
        &mut self,
        direction: ShiftDirection,
        window_size_lines: usize,
        shift_len: usize,
    ) -> usize;
    fn lines(&self) -> &VecDeque<String>;

    fn file_path(&self) -> PathBuf;

    // Get file inode number
    fn get_file_ino(&self) -> Option<u64> {
        std::fs::metadata(&self.file_path()).map(|m| m.ino()).ok()
    }
}
