use std::collections::VecDeque;
use std::fmt::Write;
use std::io::Write as IoWrite;

use log::*;

use crate::log_directory_entry::LogFileType;
use crate::log_directory_reader::DirectoryReader;
use crate::log_files::{
    live_log_file::LiveLogFile,
    log_file_trait::{LogFile, ShiftDirection},
    rotated_log_file::RotatedLogFile,
};

/// Log files registry witch implements cross-file browsing
pub struct LogRegistry {
    pub log_files: VecDeque<Box<dyn LogFile>>,
    /// Currently visible window inside [log_files]
    pub current_window: (usize, usize),
}

impl LogRegistry {
    pub fn new(log_location: &str) -> Self {
        let rotated_log_files: VecDeque<Box<dyn LogFile>> =
            DirectoryReader::read_dir_logs(log_location)
                .into_iter()
                .map(|log_entry| {
                    let mut log_file = match log_entry.log_type {
                        LogFileType::Rotated(ts) => {
                            Box::new(RotatedLogFile::new(log_entry.full_path, ts))
                                as Box<dyn LogFile>
                        }
                        LogFileType::Live => {
                            Box::new(LiveLogFile::new(log_entry.full_path)) as Box<dyn LogFile>
                        }
                    };

                    // We need to reverse log cursors since we're reading from the end of log
                    log_file.rev();
                    log_file
                })
                .collect();

        debug!("Found {} log files", rotated_log_files.len());

        let init_cursor = rotated_log_files.len() - 1;
        Self {
            log_files: rotated_log_files,
            current_window: (init_cursor, init_cursor),
        }
    }

    fn extend(&mut self, direction: ShiftDirection, windows_len: usize) -> usize {
        debug!(
            "Extend start to the {:?}, Window size: {}. Cursor: {:?}",
            direction, windows_len, self.current_window
        );

        let mut total_lines_read = 0;

        // We move end cursor to the beginning of the window and start reading logs
        // movin away the cursor to create a new window
        match direction {
            ShiftDirection::Left => {
                self.current_window.0 = self.current_window.1;
            }
            ShiftDirection::Right => {
                self.current_window.1 = self.current_window.0;
            }
        };

        while total_lines_read < windows_len {
            let cursor_to_use = match direction {
                ShiftDirection::Left => self.current_window.0,
                ShiftDirection::Right => self.current_window.1,
            };

            // Read file trying to read as much lines as we need
            let (file_lines_len, _) = self.log_files[cursor_to_use].read_and_shift(
                direction,
                windows_len - total_lines_read,
                0,
            );

            total_lines_read += file_lines_len;
            debug!(
                "Read {} lines from '{}'. Still {} to read",
                file_lines_len,
                self.log_files[cursor_to_use].file_path().display(),
                windows_len - total_lines_read
            );

            // If we've read enough lines from the current file, cursor remains the same, otherwise...
            if total_lines_read == windows_len {
                debug!("Current file had enough lines to read. Stop reading");
                break;
            }

            trace!("Extension shift window edge to the {:?}", direction);
            // ...Move corresponding cursor to expand the window
            match direction {
                ShiftDirection::Left => {
                    if self.current_window.0 == 0 {
                        debug!(
                            "End of log. Wasn't able to read {} more lines",
                            windows_len - total_lines_read
                        );
                        return total_lines_read;
                    } else {
                        self.current_window.0 -= 1;
                    }
                }
                ShiftDirection::Right => {
                    if self.current_window.1 == self.log_files.len() - 1 {
                        debug!(
                            "End of log. Wasn't able to read {} more lines",
                            windows_len - total_lines_read
                        );
                        return total_lines_read;
                    } else {
                        self.current_window.1 += 1;
                    }
                }
            }

            trace!("New cursor for extension: {:?}", self.current_window);
        }

        total_lines_read
    }

    pub fn shift(&mut self, direction: ShiftDirection, mut shift_len: usize, windows_len: usize) {
        debug!(
            "Shift start for {} lines. Window {:?}",
            shift_len, self.current_window
        );

        // Read lines needed to fill the viewport after shift
        // Reading first and shifting afterwards allows us to not shift past the end of the
        // edge files.
        // 1. In case we have enough data, total lines read will be windows_size + shift_len
        // 2. Otherwise we got less then that. Sometimes even less than a window size. In this case we don't shift
        let total_lines_read = self.extend(direction, windows_len + shift_len);

        // Shift in a way that we always have lines to fill the viewport
        shift_len = total_lines_read.saturating_sub(windows_len);
        trace!("Resulting shift len: {}", shift_len);

        // First we shift files cursors to satisfy shift len, which means we shift a single
        // file if it's big enought, or shift multiple files, if we have several files inside a
        // visible area
        while shift_len > 0 {
            let cursor_to_use = match direction {
                ShiftDirection::Left => self.current_window.1,
                ShiftDirection::Right => self.current_window.0,
            };

            debug!(
                "Shifting '{}' log for {} lines",
                self.log_files[cursor_to_use].file_path().display(),
                shift_len
            );

            let (_, lines_shifted) =
                self.log_files[cursor_to_use].read_and_shift(direction, windows_len, shift_len);

            shift_len -= lines_shifted;
            debug!(
                "Shifted for {} lines. Still {} to shift",
                lines_shifted, shift_len
            );

            // The file doesn't have enought lines to fill the viewport. Switch to the previous one
            if shift_len > 0 {
                debug!(
                    "File has not enough lines to perform shift. Still {} to shift",
                    shift_len
                );

                match direction {
                    ShiftDirection::Left => {
                        if self.current_window.1 == 0 {
                            debug!("No more files to shift");

                            break;
                        } else {
                            self.current_window.1 -= 1;

                            // Push left cursor if needed
                            self.current_window.0 =
                                std::cmp::min(self.current_window.1, self.current_window.0)
                        }
                    }
                    ShiftDirection::Right => {
                        if self.current_window.0 == self.log_files.len() - 1 {
                            debug!("No more files to shift");

                            break;
                        } else {
                            self.current_window.0 += 1;

                            // Push left cursor if needed
                            self.current_window.1 =
                                std::cmp::max(self.current_window.1, self.current_window.0)
                        }
                    }
                }
            }

            trace!("New cursor for shifting: {:?}", self.current_window);
        }
    }

    pub fn write(&self, buffer: &mut dyn Write) {
        trace!("Writing files: {:?}", self.current_window);

        for i in self.current_window.0..=self.current_window.1 {
            if self.log_files[i].lines().is_empty() {
                continue;
            }

            for line in self.log_files[i].lines() {
                if let Err(err) = buffer.write_str(line) {
                    warn!("Failed to write log into a writer {}", err.to_string());
                }
            }

            // Don't newline after the last file
            if i < self.current_window.1 {
                let _ = buffer.write_char('\n');
            }
        }
    }

    pub fn write_io(&self, buffer: &mut dyn IoWrite) {
        trace!("Writing files: {:?}", self.current_window);

        for i in self.current_window.0..=self.current_window.1 {
            if self.log_files[i].lines().is_empty() {
                continue;
            }

            for line in self.log_files[i].lines() {
                if let Err(err) = buffer.write(line.as_bytes()) {
                    eprintln!("Failed to write log into a writer {}", err.to_string());
                }
            }

            // Don't newline after the last file
            if i < self.current_window.1 {
                let _ = buffer.write("\n\r".as_bytes());
            }
        }
    }
}
