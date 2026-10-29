use crate::{exit_function, render};
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use std::io;

/// ## Editor function
/// Handles editor interface and mutates start frame vector in-place
/// Parameters
/// - start: Initial frame vector, generally should be empty
/// - x_size: size of start vector
/// - y_size: size of start\[i\] vector
/// ---
/// **Requires event stream for keyboard control and stdout for printing purposes.**
pub async fn edit_frame(
    start: &mut Vec<Vec<bool>>,
    x_size: u16,
    y_size: u16,
    reader: &mut EventStream,
    stdout: &mut io::Stdout,
) -> io::Result<()> {
    // Start cursor in center
    let mut cursor_x: u16 = x_size / 4;
    let mut cursor_y: u16 = y_size / 2;
    let mut frame: Vec<u8> = Vec::new();

    loop {
        let glyph = |i: usize, j: usize, v: bool| {
            if i == cursor_y as usize && j == cursor_x as usize {
                // Selected cell
                if v { "▒▒" } else { "▓▓" }
            } else if v {
                // Populated cell
                "██"
            } else {
                // Dead cell
                "⣏⣹"
            }
        };
        if let Err(e) = render::draw_grid(stdout, &mut frame, start, glyph) {
            exit_function::trigger_exit(stdout, 1, Some(&format!("Frame render failed: {e}")));
        }

        // Keybind system
        if let Some(Ok(event)) = reader.next().await {
            match event {
                Event::Key(key) => match key.code {
                    // Keyboard control for cursor movement via arrow keys.
                    KeyCode::Up => cursor_y = cursor_y.saturating_sub(1),
                    KeyCode::Down => {
                        if cursor_y < y_size - 1 {
                            cursor_y += 1;
                        }
                    }
                    KeyCode::Left => cursor_x = cursor_x.saturating_sub(1),
                    KeyCode::Right => {
                        if cursor_x < x_size - 1 {
                            cursor_x += 1;
                        }
                    }
                    // Toggle cell state
                    KeyCode::Enter => {
                        let cursor_y_usize: usize = cursor_y as usize;
                        let cursor_x_usize: usize = cursor_x as usize;
                        start[cursor_y_usize][cursor_x_usize] =
                            !start[cursor_y_usize][cursor_x_usize];
                    }
                    // Space to start playing simulation.
                    KeyCode::Char(' ') => break, // Exit edit mode and start simulation
                    KeyCode::Char('q') => {
                        exit_function::trigger_exit(stdout, 0, None);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Ok(())
}
