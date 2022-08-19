use std::collections::VecDeque;

use karo_log_viewer::{log_file_trait::ShiftDirection, log_window::LogWindow};

#[test]
fn test_window() {
    let mut window = LogWindow::new();

    // [0, 1]
    let mut window_size = window.shift(
        ShiftDirection::Right,
        2,
        vec!["test0".into(), "test1".into()],
    );

    assert_eq!(window_size, 2);
    assert_eq!(window.start_cursor(), 0);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(
        window.lines(),
        &VecDeque::from_iter(["test0".to_owned(), "test1".to_owned()])
    );

    // [x, 1, 2]
    window_size = window.shift(ShiftDirection::Right, 1, vec!["test2".into()]);

    assert_eq!(window_size, 2);
    assert_eq!(window.start_cursor(), 6);
    assert_eq!(window.end_cursor(), 18);
    assert_eq!(
        window.lines(),
        &VecDeque::from_iter(["test1".to_owned(), "test2".to_owned()])
    );

    // [0, 1]
    window_size = window.shift(ShiftDirection::Left, 1, vec!["test0".into()]);

    assert_eq!(window_size, 2);
    assert_eq!(window.start_cursor(), 0);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(
        window.lines(),
        &VecDeque::from_iter(["test0".to_owned(), "test1".to_owned()])
    );

    // [x, 1]
    window_size = window.shift(ShiftDirection::Right, 1, vec![]);

    assert_eq!(window_size, 1);
    assert_eq!(window.start_cursor(), 6);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(window.lines(), &VecDeque::from_iter(["test1".to_owned()]));

    // [x, x]
    window_size = window.shift(ShiftDirection::Right, 1, vec![]);

    assert_eq!(window_size, 0);
    assert_eq!(window.start_cursor(), 12);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(window.lines(), &VecDeque::new());
}
