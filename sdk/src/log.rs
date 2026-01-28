use std::{
    fmt::Display,
    io::{Stdout, Write},
};

use anstyle::AnsiColor;

#[derive(PartialEq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

pub struct Log {
    stdout: Stdout,
    log_level: Verbosity,
}

impl Log {
    pub fn new(log_level: Verbosity) -> Self {
        Self {
            stdout: std::io::stdout(),
            log_level,
        }
    }

    pub fn print_normal<T, U>(&mut self, label: T, label_color: AnsiColor, message: U)
    where
        T: Display,
        U: Display,
    {
        self.print_normal_with(label, label_color, || &message);
    }

    pub fn print_normal_with<T, U, F>(&mut self, label: T, label_color: AnsiColor, msg_fn: F)
    where
        T: Display,
        U: Display,
        F: Fn() -> U,
    {
        if self.log_level != Verbosity::Quiet {
            writeln!(
                self.stdout,
                "{}{:>12}{} {}",
                anstyle::Style::new()
                    .fg_color(Some(anstyle::Color::Ansi(label_color)))
                    .bold()
                    .render(),
                label,
                anstyle::Reset,
                msg_fn()
            )
            .expect("error writing to terminal");
        }
    }

    pub fn print_verbose<T, U>(&mut self, label: T, label_color: AnsiColor, message: U)
    where
        T: Display,
        U: Display,
    {
        self.print_verbose_with(label, label_color, || &message);
    }

    pub fn print_verbose_with<T, U, F>(&mut self, label: T, label_color: AnsiColor, msg_fn: F)
    where
        T: Display,
        U: Display,
        F: Fn() -> U,
    {
        if self.log_level == Verbosity::Verbose {
            writeln!(
                self.stdout,
                "{}{:>12}{} {}",
                anstyle::Style::new()
                    .fg_color(Some(anstyle::Color::Ansi(label_color)))
                    .bold()
                    .render(),
                label,
                anstyle::Reset,
                msg_fn()
            )
            .expect("error writing to terminal");
        }
    }

    pub fn status<T, U>(&mut self, label: T, message: U)
    where
        T: Display,
        U: Display,
    {
        self.status_with(label, || &message);
    }

    pub fn status_with<T, U, F>(&mut self, label: T, msg_fn: F)
    where
        T: Display,
        U: Display,
        F: Fn() -> U,
    {
        self.print_normal_with(label, AnsiColor::Green, msg_fn);
    }

    pub fn action<T, U>(&mut self, label: T, message: U)
    where
        T: Display,
        U: Display,
    {
        self.action_with(label, || &message);
    }

    pub fn action_with<T, U, F>(&mut self, label: T, msg_fn: F)
    where
        T: Display,
        U: Display,
        F: Fn() -> U,
    {
        self.print_normal_with(label, AnsiColor::BrightCyan, msg_fn);
    }

    pub fn error<T>(&mut self, message: T)
    where
        T: Display,
    {
        self.error_with(|| &message);
    }

    pub fn error_with<T, F>(&mut self, msg_fn: F)
    where
        T: Display,
        F: Fn() -> T,
    {
        self.print_normal_with("Error", AnsiColor::Red, msg_fn);
    }

    pub fn warn<T>(&mut self, message: T)
    where
        T: Display,
    {
        self.warn_with(|| &message);
    }

    pub fn warn_with<T, F>(&mut self, msg_fn: F)
    where
        T: Display,
        F: Fn() -> T,
    {
        self.print_normal_with("Warning", AnsiColor::Yellow, msg_fn);
    }
}
