use std::collections::VecDeque;

use crate::log_file_trait::ShiftDirection;

/// Log file window.
/// [start_cursor] and [end_cursor] always point to the beginning of the [lines]
pub struct LogWindow {
    /// Cursor at the top of the window
    start_cursor: usize,
    /// Cursor at the top of the window
    end_cursor: usize,
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

    pub fn start_cursor(&self) -> usize {
        self.start_cursor
    }

    pub fn end_cursor(&self) -> usize {
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
                        self.end_cursor -= line.len() + 1 // \n
                    } else {
                        break;
                    }
                }

                for aline in append_lines.into_iter() {
                    self.start_cursor -= aline.len() + 1;
                    self.lines.push_front(aline);
                }
            }
            ShiftDirection::Right => {
                for _ in 0..shift_len {
                    if let Some(line) = self.lines.pop_front() {
                        self.start_cursor += line.len() + 1 // \n
                    } else {
                        break;
                    }
                }

                for aline in append_lines.into_iter() {
                    self.end_cursor += aline.len() + 1;
                    self.lines.push_back(aline);
                }
            }
        }

        self.lines.len()
    }
}
