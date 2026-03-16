mod cells; // Contains conway's game of life next frame gen for 2d vector

use std::io;
use std::thread;
use std::time::Duration; // Frame timers
use crossterm::{execute, cursor::Hide, terminal}; // Handle TUI and cursor hide

fn main() {
    // Try to hide cursor
    execute!(io::stdout(), Hide).expect("Could not hide cursor...");
    
    // Get terminal size
    let (x,y) = terminal::size().expect("Terminal size not detected!\n");
    let x_usize = x as usize;
    let y_usize = y as usize;


    // Fill 2D vector with random boolean values
    let mut start:Vec<Vec<bool>> = vec![vec![false; x_usize];y_usize];
    for i in 0..y_usize {
        for j in 0..x_usize {
            start[i][j] = rand::random_bool(0.5);
        }
    }

    // Multithread loop through frames.
    let render_thread = thread::spawn(move || {
        loop {
            print!("\x1B[2J");
            for i in 0..y_usize {
                let buf = start[i].iter().map(|&v| if v {"█"} else {" "}).collect::<String>();
                print!("{}\n",buf);
            }
            start = cells::calculate_next(start);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Join thread
    render_thread.join().expect("Failed to join render thread");
}
