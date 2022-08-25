use std::collections::VecDeque;

use log::LevelFilter;

use karo_log_viewer::log_files::{log_file_trait::ShiftDirection, log_window::LogWindow};

#[test]
fn test_window() {
    let _ = pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Trace)
        .try_init();

    let mut window = LogWindow::new();

    // [0, 1]
    let mut shift_len = window.shift(
        ShiftDirection::Right,
        2,
        vec!["test0\n".into(), "test1\n".into()],
    );

    assert_eq!(shift_len, 0);
    assert_eq!(window.start_cursor(), 0);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(
        window.lines(),
        &VecDeque::from_iter(["test0\n".to_owned(), "test1\n".to_owned()])
    );

    // [x, 1, 2]
    shift_len = window.shift(ShiftDirection::Right, 1, vec!["test2".into()]);

    assert_eq!(shift_len, 1);
    assert_eq!(window.start_cursor(), 6);
    assert_eq!(window.end_cursor(), 17);
    assert_eq!(
        window.lines(),
        &VecDeque::from_iter(["test1\n".to_owned(), "test2".to_owned()])
    );

    // [0, 1]
    shift_len = window.shift(ShiftDirection::Left, 1, vec!["test0\n".into()]);

    assert_eq!(shift_len, 1);
    assert_eq!(window.start_cursor(), 0);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(
        window.lines(),
        &VecDeque::from_iter(["test0\n".to_owned(), "test1\n".to_owned()])
    );

    // [x, 1]
    shift_len = window.shift(ShiftDirection::Right, 1, vec![]);

    assert_eq!(shift_len, 1);
    assert_eq!(window.start_cursor(), 6);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(window.lines(), &VecDeque::from_iter(["test1\n".to_owned()]));

    // [x, x]
    shift_len = window.shift(ShiftDirection::Right, 1, vec![]);

    assert_eq!(shift_len, 1);
    assert_eq!(window.start_cursor(), 12);
    assert_eq!(window.end_cursor(), 12);
    assert_eq!(window.lines(), &VecDeque::new());

    // [x, x, x]
    window.rev(17);
    assert_eq!(shift_len, 1);
    assert_eq!(window.start_cursor(), 17);
    assert_eq!(window.end_cursor(), 17);
    assert_eq!(window.lines(), &VecDeque::new());

    // [x, x, 2]
    shift_len = window.shift(ShiftDirection::Left, 1, vec!["test2".into()]);

    assert_eq!(shift_len, 0);
    assert_eq!(window.start_cursor(), 12);
    assert_eq!(window.end_cursor(), 17);
    assert_eq!(window.lines(), &VecDeque::from_iter(["test2".to_owned()]));

    // [0, 1, 2]
    shift_len = window.shift(
        ShiftDirection::Left,
        0,
        vec!["test0\n".into(), "test1\n".into()],
    );

    assert_eq!(shift_len, 0);
    assert_eq!(window.start_cursor(), 0);
    assert_eq!(window.end_cursor(), 17);
    assert_eq!(
        window.lines(),
        &VecDeque::from_iter([
            "test0\n".to_owned(),
            "test1\n".to_owned(),
            "test2".to_owned()
        ])
    );

    // [2]
    shift_len = window.shift(ShiftDirection::Right, 2, vec![]);

    assert_eq!(shift_len, 2);
    assert_eq!(window.start_cursor(), 12);
    assert_eq!(window.end_cursor(), 17);
    assert_eq!(window.lines(), &VecDeque::from_iter(["test2".to_owned()]));
}
