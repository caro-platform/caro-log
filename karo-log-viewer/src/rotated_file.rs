use std::{
    collections::VecDeque,
    fs::File as FsFile,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use log::*;

use crate::file_trait::{File, ShiftDirection};

// Chunk size which we read first and than split into lines
const READ_CHUNK_SIZE_BYTES: u64 = 1_000;

pub struct RotatedFile {
    file_path: PathBuf,
    handle: Option<FsFile>,
    cursor_pos: u64,
    file_len: u64,
}

impl RotatedFile {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            handle: None,
            cursor_pos: 0,
            file_len: 0,
        }
    }

    fn open_log_file(&mut self, direction: ShiftDirection) -> Option<()> {
        self.handle = Some(FsFile::open(&self.file_path).ok()?);

        self.file_len = self.file_len()?;

        // Reset cursor position
        // If we want to shift down, set cursor pos to the beginning of the file, otherwise to the end
        self.cursor_pos = if direction == ShiftDirection::Down {
            0
        } else {
            self.file_len
        };

        Some(())
    }

    fn close_log_file(&mut self) {
        self.handle = None
    }

    fn file_len(&self) -> Option<u64> {
        self.handle
            .as_ref()
            .and_then(|f| f.metadata().ok())
            .map(|m| m.len())
    }

    /// Clamps reading chunk inside the file
    fn get_chunk_to_read(&self, direction: ShiftDirection) -> (u64, u64) {
        match direction {
            ShiftDirection::Down => {
                let left = self.cursor_pos;
                let right = std::cmp::min(self.file_len, self.cursor_pos + READ_CHUNK_SIZE_BYTES);
                (left, right)
            }
            ShiftDirection::Up => {
                let left = std::cmp::max(0, self.cursor_pos as i64 - READ_CHUNK_SIZE_BYTES as i64);
                let right = self.cursor_pos;
                (left as u64, right)
            }
        }
    }

    /// Split buffer into lines
    // This is a tricky one. We not only split into lines, but also eliminate partial lines, and
    // Update cursor position
    fn split_buffer(
        &mut self,
        read_buf: String,
        mut chunk_start: u64,
        mut chunk_end: u64,
        direction: ShiftDirection,
    ) -> Vec<String> {
        let first_partial_line = chunk_start != 0 && read_buf.find('\n').is_some();
        let last_partial_line = chunk_end != self.file_len && read_buf.rfind('\n').is_some();

        let mut lines: VecDeque<String> = read_buf.lines().map(|l| l.into()).collect();

        if first_partial_line {
            // If we have partial line at the beginning of the chunk, we update true chunk start to the newline
            // to set good cursor position
            if let Some(line) = lines.pop_front() {
                debug!("Partial line at the beginning of the chunk: '{}'", line);

                chunk_start += line.len() as u64 + 1 // \n
            }
        }

        if last_partial_line {
            // If we have partial line at the end of the chunk, we update true chunk end to the newline
            // to set good cursor position
            if let Some(line) = lines.pop_back() {
                debug!("Partial line at the end of the chunk: '{}'", line);

                chunk_end -= line.len() as u64 + 1 // \n
            }
        }

        // Update cursor position to the new newline
        self.cursor_pos = match direction {
            ShiftDirection::Up => chunk_start,
            ShiftDirection::Down => chunk_end,
        };

        debug!("New cursor position: {}", self.cursor_pos);

        lines.into_iter().collect()
    }

    /// Read lines inside the file chunk
    fn read_lines(&mut self, direction: ShiftDirection) -> Vec<String> {
        let (mut chunk_start, chunk_end) = self.get_chunk_to_read(direction);
        debug!("Reading chunk of bytes <{}, {}>", chunk_start, chunk_end);

        assert!(chunk_start <= chunk_end);

        // No more data to read
        if chunk_start == chunk_end {
            return vec![];
        }

        // Shift left chunk start to identify if we have a beginning of the line
        if chunk_start > 0 {
            chunk_start -= 1
        }

        if self.handle.is_none() {
            error!("Trying to log with closed handle");
            return vec![];
        }

        let mut handle = self.handle.as_ref().unwrap();

        if handle.seek(SeekFrom::Start(chunk_start)).is_err() {
            warn!("Failed to seek log file");
            return vec![];
        }

        let mut read_buf = String::new();
        if handle
            .take(chunk_end - chunk_start)
            .read_to_string(&mut read_buf)
            .is_err()
        {
            warn!("Failed to read log file");
            return vec![];
        }

        // Split read buffer into lines
        self.split_buffer(read_buf, chunk_start, chunk_end, direction)
    }
}

impl File for RotatedFile {
    fn shift_and_read(
        &mut self,
        direction: ShiftDirection,
        window_size_lines: usize,
    ) -> Vec<String> {
        // We'll substract number of read lines, so we want it signed
        let mut window_size_lines = window_size_lines as isize;

        debug!(
            "Reading log file '{}' {:?}",
            self.file_path.display(),
            direction
        );
        let mut result = vec![];

        // If handle is none we just opening this rotated file or returning from it's neighbour logs
        if self.handle.is_none() {
            if self.open_log_file(direction).is_none() {
                warn!(
                    "Failed to open rotated file at '{}'",
                    self.file_path.display(),
                );
                return result;
            }
        }

        // Keep reading until get needed number of lines
        while window_size_lines > 0 {
            let new_lines = self.read_lines(direction);

            // No more lines to read in the file break the loop, return whatever we've read so far
            if new_lines.len() == 0 {
                debug!("Reached EOF");
                // If we didn't read data from the file, it went out of the range of interest. Close it for a while
                if result.is_empty() {
                    debug!("No lines read. Out of the file range");

                    self.close_log_file()
                }

                break;
            }

            window_size_lines -= new_lines.len() as isize;
            debug!(
                "Read {} lines of log. {} to read",
                new_lines.len(),
                window_size_lines
            );

            result.extend(new_lines.into_iter());
        }

        result
    }
}
