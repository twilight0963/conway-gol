use crossterm::{cursor, execute, terminal};
use std::io;

/// Restores the terminal: shows the cursor, leaves the alternate screen and raw mode.
/// Errors are ignored since this runs on the way out.
pub fn restore_terminal(stdout: &mut io::Stdout) {
    let _ = execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

/// Restores the terminal, prints msg (if any) to stderr, then exits.
pub fn trigger_exit(stdout: &mut io::Stdout, exit_code: i32, msg: Option<&str>) -> ! {
    restore_terminal(stdout);
    if let Some(msg) = msg {
        eprintln!("{msg}");
    }
    std::process::exit(exit_code);
}
