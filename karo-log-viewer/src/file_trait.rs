#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShiftDirection {
    Up,
    Down,
}

pub trait File {
    fn shift_and_read(
        &mut self,
        direction: ShiftDirection,
        window_size_lines: usize,
    ) -> Vec<String>;
}
