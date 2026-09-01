use crossterm::{
    execute,
    terminal::{self},
};
use std::io;

pub fn trigger_exit(stdout: &mut io::Stdout, exit_code: i32) {
    if execute!(stdout, terminal::LeaveAlternateScreen).is_err() {
        eprintln!("Unable to exit alternate screen");
    }
    std::process::exit(exit_code);
}
