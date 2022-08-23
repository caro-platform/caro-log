use std::{
    io::{stdout, Stdout, Write},
    ops::Not,
};

use termion::screen::{AlternateScreen, ToAlternateScreen, ToMainScreen};

#[derive(Clone, Copy)]
enum ActiveScreen {
    Main,
    Alternate,
}

impl Not for ActiveScreen {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ActiveScreen::Main => ActiveScreen::Alternate,
            ActiveScreen::Alternate => ActiveScreen::Main,
        }
    }
}

pub struct Screens {
    active: ActiveScreen,
    main_screen: Stdout,
    alt_screen: AlternateScreen<Stdout>,
}

impl Screens {
    pub fn new() -> Self {
        Self {
            active: ActiveScreen::Main,
            main_screen: stdout(),
            alt_screen: AlternateScreen::from(stdout()),
        }
    }

    pub fn height() -> u16 {
        termion::terminal_size().map(|(_w, h)| h).unwrap_or(20)
    }

    pub fn write(&mut self) -> &mut dyn Write {
        // Retun non-visible screen
        let result = match self.active {
            ActiveScreen::Main => &mut self.alt_screen,
            ActiveScreen::Alternate => &mut self.main_screen,
        };

        if let Err(err) = write!(
            result,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        ) {
            eprintln!("Failed to clear alternate screen: {}", err.to_string());
        }

        result
    }

    pub fn switch(&mut self) {
        let result = match self.active {
            ActiveScreen::Main => write!(self.alt_screen, "{}", ToAlternateScreen),
            ActiveScreen::Alternate => write!(self.alt_screen, "{}", ToMainScreen),
        }
        .and_then(|_| self.alt_screen.flush());

        if let Err(err) = result {
            eprintln!("Failed to swap screens: {}", err.to_string())
        } else {
            self.active = !self.active;
        }
    }
}
