use std::{collections::VecDeque, os::unix::prelude::MetadataExt, path::PathBuf};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShiftDirection {
    Left,
    Right,
}

pub trait LogFile {
    /// Shift windows position and read enought lines to keep windown of **windows_size_lines** size
    /// **Returns** (lines in the window left, number of shifted lines)
    /// Number of shifted lines can be used when with shift viewport. In this case if the file goes
    /// out of the viewport, we want to know how many lines we still need to shift in the following file
    fn read_and_shift(
        &mut self,
        direction: ShiftDirection,
        window_size_lines: usize,
        shift_len: usize,
    ) -> (usize, usize);
    fn lines(&self) -> &VecDeque<String>;

    fn reset(&mut self);

    fn rev(&mut self);

    fn file_path(&self) -> PathBuf;

    // Get file inode number
    fn get_file_ino(&self) -> Option<u64> {
        std::fs::metadata(&self.file_path()).map(|m| m.ino()).ok()
    }
}
