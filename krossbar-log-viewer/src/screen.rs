use std::io::{stdout, Stdout, Write};

use termion::{
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen, ToAlternateScreen, ToMainScreen},
};

/// Screen to display logs. Uses Termion alt screen to act as an app
pub struct Screen {
    alt_screen: AlternateScreen<RawTerminal<Stdout>>,
}

impl Screen {
    pub fn new() -> Self {
        let mut alt_screen = stdout()
            .into_raw_mode()
            .unwrap()
            .into_alternate_screen()
            .unwrap();

        if let Err(err) = write!(alt_screen, "{}{}", termion::cursor::Hide, ToAlternateScreen) {
            eprintln!("Failed to switch to alternate screen: {}", err.to_string());
        }

        if let Err(err) = alt_screen.flush() {
            eprint!("Failed to flush alternate screen: {}", err.to_string())
        }

        Self { alt_screen }
    }

    pub fn size() -> (u16, u16) {
        termion::terminal_size().unwrap_or((80, 20))
    }

    pub fn write<'a>(&'a mut self) -> WriteHandle<'a> {
        if let Err(err) = write!(
            self.alt_screen,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        ) {
            eprintln!("Failed to clear alternate screen: {}", err.to_string());
        }

        WriteHandle {
            screen: &mut self.alt_screen,
        }
    }
}

/// Drop implementation, which restores primary TTY if exited
impl Drop for Screen {
    fn drop(&mut self) {
        let _ = write!(self.alt_screen, "{}{}", ToMainScreen, termion::cursor::Show);
    }
}

pub struct WriteHandle<'a> {
    pub screen: &'a mut AlternateScreen<RawTerminal<Stdout>>,
}

impl<'a> Write for WriteHandle<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.screen.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.screen.flush()
    }
}

impl<'a> Drop for WriteHandle<'a> {
    fn drop(&mut self) {
        if let Err(err) = self.screen.flush() {
            eprint!("Failed to flush screen: {}", err.to_string())
        }
    }
}
