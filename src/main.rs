mod cells; // Contains conway's game of life next frame gen for 2d vector

use std::io;
use crossterm::{
    execute, queue,
    style::{self, Stylize}, cursor, terminal
};
use tokio::time::Duration;


#[tokio::main]
async fn main() {
    let mut stdout = io::stdout();
    // Try to hide cursor
    execute!(stdout, cursor::Hide).expect("Could not hide cursor...");
    
    // Get terminal size
    let (x,y) = terminal::size().expect("Terminal size not detected!\n");
    let x_usize = x as usize;
    let y_usize = y as usize;

    // Fill 2D vector with random boolean values
    let mut start:Vec<Vec<bool>> = vec![vec![false; x_usize];y_usize];
    for i in 0..y_usize {
        for j in 0..x_usize {
            start[i][j] = rand::random_bool(0.1);
        }
    }

    // Multithread loop through frames.
    tokio::spawn(async move {
        loop {
            let mut neighbors = cells::get_all_neighbors(&start,x_usize,y_usize);
            // Clear screen before starting print.
            execute!(stdout, terminal::Clear(terminal::ClearType::All)).expect("Could not clear terminal!");

            // Print by line
            for i in 0..y {
                let buf = start[i as usize].iter().map(|&v| if v {"█"} else {" "}).collect::<String>(); // Map true and false to lit up cell and whitespace.
                queue!(stdout, cursor::MoveTo(0,i), style::PrintStyledContent(buf.blue())).expect("Failed to print!"); // Queue to print.
            }

            // Start calculating next and wait atleast 70ms
            neighbors = cells::calculate_next(&mut start,neighbors,y_usize,x_usize);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    // To ensure program doesn't exit before first iteration
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
