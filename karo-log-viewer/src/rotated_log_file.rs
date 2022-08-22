use std::{
    collections::VecDeque,
    fs::File as FsFile,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use chrono::NaiveDateTime;
use log::*;

use crate::{
    log_file_trait::{LogFile, ShiftDirection},
    log_window::LogWindow,
};

// Chunk size which we read first and than split into lines
const READ_CHUNK_SIZE_BYTES: u64 = 1_000;

pub struct RotatedLogFile {
    file_path: PathBuf,
    timestamp: NaiveDateTime,
    handle: Option<FsFile>,
    file_len: u64,
    window: LogWindow,
}

impl RotatedLogFile {
    pub fn new(file_path: PathBuf, timestamp: NaiveDateTime) -> Self {
        Self {
            file_path,
            timestamp,
            handle: None,
            file_len: 0,
            window: LogWindow::new(),
        }
    }

    pub fn timestamp(&self) -> &NaiveDateTime {
        &self.timestamp
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
                // Windows end cursor can past the end of the file, because window assumes
                // that every line end with a newline character
                let left = std::cmp::min(self.file_len, self.window.end_cursor());
                let right = std::cmp::min(
                    self.file_len,
                    self.window.end_cursor() + READ_CHUNK_SIZE_BYTES,
                );
                (left, right)
            }
            ShiftDirection::Left => {
                let left = std::cmp::max(
                    0,
                    self.window.start_cursor() as i64 - READ_CHUNK_SIZE_BYTES as i64,
                );
                let right = self.window.start_cursor();
                (left as u64, right)
            }
        }
    }

    /// Split buffer into lines
    // This is a tricky one. We not only split into lines, but also eliminate partial lines, and
    // drop extra ones
    fn split_buffer(
        &mut self,
        read_buf: String,
        chunk_start: u64,
        chunk_end: u64,
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
            lines.pop_front();
        }

        if last_partial_line {
            lines.pop_back();
        }

        let lines_len = lines.len();

        // Drop extra lines
        match direction {
            ShiftDirection::Left => {
                if lines_len > num_lines {
                    lines.into_iter().skip(lines_len - num_lines).collect()
                } else {
                    lines.into()
                }
            }
            ShiftDirection::Right => lines.into_iter().take(num_lines).collect(),
        }
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

    fn shrink_window(&mut self, direction: ShiftDirection, num_lines: usize) {
        debug!(
            "Current window is too big. Shrinking it to the {:?} for {} lines",
            direction, num_lines
        );

        // Actually direction of shrinking os opposite to expanding
        match direction {
            ShiftDirection::Left => self.window.shift(ShiftDirection::Right, num_lines, vec![]),
            ShiftDirection::Right => self.window.shift(ShiftDirection::Left, num_lines, vec![]),
        };
    }
}

impl LogFile for RotatedLogFile {
    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn lines(&self) -> &VecDeque<String> {
        self.window.lines()
    }

    fn reset(&mut self) {
        self.window.reset()
    }

    fn rev(&mut self) {
        if self.handle.is_none() {
            self.open_log_file();
        }

        self.window.rev(self.file_len + 1); // phantom \n
    }

    fn read_and_shift(
        &mut self,
        direction: ShiftDirection,
        window_size_lines: usize,
        shift_len: usize,
    ) -> (usize, usize) {
        debug!(
            "Reading log file '{}' {:?}",
            self.file_path.display(),
            direction
        );

        // Calculate number of rows to read. This is basically new windows size + shift
        let mut lines_to_read =
            (window_size_lines + shift_len) as isize - self.window.len() as isize;
        debug!("Going to read {} new lines", lines_to_read);

        // If windows is bigger that we need, shrink it
        if lines_to_read < 0 {
            self.shrink_window(direction, -lines_to_read as usize);
        }

        // If handle is none we just opening this rotated file or returning from it's neighbour logs
        if self.handle.is_none() {
            if self.open_log_file().is_none() {
                warn!("Failed to open log file at '{}'", self.file_path.display(),);
                return (0, 0);
            }
        }

        // Keep reading until get needed number of lines
        while lines_to_read > 0 {
            let new_lines = self.read_lines(direction, lines_to_read as usize);

            // No more lines to read in the file break the loop, return whatever we've read so far
            if new_lines.len() == 0 {
                debug!("Reached EOF");

                // If we didn't read data from the file, it went out of the range of interest. Close it for a while
                if self.window.len() == 0 {
                    debug!(
                        "No lines left in the window. Out of the file range. Closing the log file"
                    );

                    self.close_log_file()
                }

                break;
            }

            let new_lines_size = new_lines.len();
            lines_to_read -= new_lines_size as isize;

            // Add newly readed lines to the window
            self.window.shift(direction, 0, new_lines);

            debug!(
                "Read {} lines of log. {} to read. New window size: {}",
                new_lines_size,
                lines_to_read,
                self.window.len()
            );
        }

        // Shift window to drop line we won't need anymore
        let lines_shifted = self.window.shift(direction, shift_len, vec![]);

        (self.window.len(), lines_shifted)
    }
}
