use std::collections::VecDeque;

use log::*;

use crate::log_file_trait::ShiftDirection;

/// Log file window.
/// [start_cursor] and [end_cursor] always point to the beginning of the [lines]
pub struct LogWindow {
    /// Cursor at the top of the window
    start_cursor: u64,
    /// Cursor at the top of the window
    end_cursor: u64,
    /// Log lines inside of the window
    lines: VecDeque<String>,
}

impl LogWindow {
    pub fn new() -> Self {
        LogWindow {
            start_cursor: 0,
            end_cursor: 0,
            lines: VecDeque::new(),
        }
    }

    pub fn rev(&mut self, end_pos: u64) {
        self.start_cursor = end_pos;
        self.end_cursor = end_pos;
        self.lines = VecDeque::new();
    }

    pub fn start_cursor(&self) -> u64 {
        self.start_cursor
    }

    pub fn end_cursor(&self) -> u64 {
        self.end_cursor
    }

    pub fn lines(&self) -> &VecDeque<String> {
        &self.lines
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Shift window prepending lines if direction is Left, or appending otherwise.
    /// After appending lines strcture will shift window towards the direction of shift.
    /// If **append_size.len() < shift_len** window shrinks
    /// **Returns** size of the new window
    pub fn shift(
        &mut self,
        direction: ShiftDirection,
        shift_len: usize,
        append_lines: Vec<String>,
    ) -> usize {
        match direction {
            ShiftDirection::Left => {
                for _ in 0..shift_len {
                    if let Some(line) = self.lines.pop_back() {
                        self.end_cursor -= line.len() as u64 + 1 // \n
                    } else {
                        break;
                    }
                }

                for aline in append_lines.into_iter().rev() {
                    self.start_cursor -= aline.len() as u64 + 1;
                    self.lines.push_front(aline);
                }
            }
            ShiftDirection::Right => {
                for _ in 0..shift_len {
                    if let Some(line) = self.lines.pop_front() {
                        self.start_cursor += line.len() as u64 + 1 // \n
                    } else {
                        break;
                    }
                }

                for aline in append_lines.into_iter() {
                    self.end_cursor += aline.len() as u64 + 1;
                    self.lines.push_back(aline);
                }
            }
        }

        debug!(
            "New cursor position: <{}, {}>",
            self.start_cursor, self.end_cursor
        );

        self.lines.len()
    }
}
