use std::{
    collections::VecDeque,
    fs::File as FsFile,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use log::*;

use crate::log_file_trait::{LogFile, ShiftDirection};

// Chunk size which we read first and than split into lines
const READ_CHUNK_SIZE_BYTES: u64 = 1_000;

pub struct LiveLogFile {
    file_path: PathBuf,
    handle: Option<FsFile>,
    cursor_pos: u64,
    file_len: u64,
    lines: VecDeque<String>,
}

impl LiveLogFile {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            handle: None,
            cursor_pos: 0,
            file_len: 0,
            lines: VecDeque::new(),
        }
    }

    pub fn rev(&mut self) {
        if self.handle.is_none() {
            self.open_log_file();
        }

        self.cursor_pos = self.file_len;
    }

    fn open_log_file(&mut self) -> Option<()> {
        self.handle = Some(FsFile::open(&self.file_path).ok()?);

        self.file_len = self.file_len()?;

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
            ShiftDirection::Right => {
                let left = self.cursor_pos;
                let right = std::cmp::min(self.file_len, self.cursor_pos + READ_CHUNK_SIZE_BYTES);
                (left, right)
            }
            ShiftDirection::Left => {
                let left = std::cmp::max(0, self.cursor_pos as i64 - READ_CHUNK_SIZE_BYTES as i64);
                let right = self.cursor_pos;
                (left as u64, right)
            }
        }
    }

    /// The function will use iterator to collect new deque and count how make bytes resulting
    /// lines consist of
    fn truncate_and_count(iter: impl Iterator<Item = String>) -> (VecDeque<String>, usize) {
        iter.fold((VecDeque::new(), 0), |(mut lines_acc, len), l| {
            let new_len = len + l.len() + 1; // \n
            lines_acc.push_back(l);
            (lines_acc, new_len)
        })
    }

    /// Split buffer into lines
    // This is a tricky one. We not only split into lines, but also eliminate partial lines, and
    // Update cursor position
    fn split_buffer(
        &mut self,
        read_buf: String,
        mut chunk_start: u64,
        mut chunk_end: u64,
        num_lines: usize,
        direction: ShiftDirection,
    ) -> Vec<String> {
        let first_partial_line = chunk_start != 0 && read_buf.find('\n').is_some();
        // Note, we have last partial lines only if going down
        let last_partial_line = direction == ShiftDirection::Right
            && chunk_end != self.file_len
            && read_buf.rfind('\n').is_some();

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

        // Set cursor position at edge of the read chunk
        // If we've read not enought lines, set cursor position at the end or beginnning of chunk
        if lines.len() <= num_lines {
            self.cursor_pos = match direction {
                ShiftDirection::Right => chunk_end,
                ShiftDirection::Left => chunk_start,
            }
        // Else drop extra lines and update cursor to be set to the edge of the lines we'll return
        } else {
            (lines, self.cursor_pos) = match direction {
                // If reading downwards, take first N lines
                ShiftDirection::Right => {
                    let (trunkated_lines, total_bytes) =
                        Self::truncate_and_count(lines.into_iter().take(num_lines));
                    (trunkated_lines, chunk_start + total_bytes as u64)
                }
                // Otherwise last N lines
                ShiftDirection::Left => {
                    let lines_to_skip = lines.len() - num_lines;

                    let (trunkated_lines, total_bytes) =
                        Self::truncate_and_count(lines.into_iter().skip(lines_to_skip));
                    (trunkated_lines, chunk_end - total_bytes as u64)
                }
            };
        }

        debug!(
            "Total lines read: {}. New cursor position: {}",
            lines.len(),
            self.cursor_pos
        );

        lines.into_iter().collect()
    }

    /// Read lines inside the file chunk
    fn read_lines(&mut self, direction: ShiftDirection, num_lines: usize) -> Vec<String> {
        let (mut chunk_start, chunk_end) = self.get_chunk_to_read(direction);
        debug!("Reading chunk of bytes <{}, {}>", chunk_start, chunk_end);

        assert!(chunk_start <= chunk_end);

        // No more data to read
        if chunk_start == chunk_end {
            return vec![];
        }

        // Shift chunk start to the left, and chunk end to the right to identify if we have a beginning of the line,
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
        self.split_buffer(read_buf, chunk_start, chunk_end, num_lines, direction)
    }
}

impl LogFile for LiveLogFile {
    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn lines(&self) -> &VecDeque<String> {
        &self.lines
    }

    fn shift_and_read(
        &mut self,
        direction: ShiftDirection,
        window_size_lines: usize,
        _shift_len: usize,
    ) -> usize {
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
            if self.open_log_file().is_none() {
                warn!(
                    "Failed to open rotated file at '{}'",
                    self.file_path.display(),
                );
                return result.len();
            }
        }

        // Keep reading until get needed number of lines
        while window_size_lines > 0 {
            let new_lines = self.read_lines(direction, window_size_lines as usize);

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

        result.len()
    }
}
