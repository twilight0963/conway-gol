use crossterm::{
    cursor, queue,
    style::{Color, ResetColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Draws the whole grid with a single write to stdout, avoiding tearing.
/// - frame: reusable byte buffer, cleared on every call
/// - glyph: maps (row, col, alive) to the two-character cell representation
pub fn draw_grid(
    stdout: &mut io::Stdout,
    frame: &mut Vec<u8>,
    grid: &[Vec<bool>],
    glyph: impl Fn(usize, usize, bool) -> &'static str,
) -> io::Result<()> {
    frame.clear();
    queue!(frame, SetForegroundColor(Color::Blue))?;
    for (i, row) in grid.iter().enumerate() {
        queue!(frame, cursor::MoveTo(0, i as u16))?;
        for (j, &alive) in row.iter().enumerate() {
            frame.extend_from_slice(glyph(i, j, alive).as_bytes());
        }
    }
    queue!(frame, ResetColor)?;
    stdout.write_all(frame)?;
    stdout.flush()
}
