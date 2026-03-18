mod cells; // Contains conway's game of life next frame gen for 2d vector

use std::io;
use crossterm::{
    execute, 
    style::{self, Stylize}, cursor, terminal,
    event::{self, Event, KeyCode,KeyEvent,KeyEventKind}
};
use tokio::time::Duration;

pub fn read_char() -> std::io::Result<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            modifiers: _,
            state: _,
        })) = event::read()
        {
            return Ok(c);
        }
    }
}

#[tokio::main]
async fn main() {
    const FPS:u64 = 24; // Frames per second for cell rendering.
    const INIT_POPULATION:f64 = 0.1; // Starting chance of a cell being active.
    let mut stdout = io::stdout();
    // Try to hide cursor
    if execute!(stdout, cursor::Hide).is_err(){
        println!("Unable to hide cursor!");
    }
    
    // Get terminal size
    let (x,y) = terminal::size().expect("Terminal size not detected! Exiting...\n");
    let x_usize = x as usize;
    let y_usize = y as usize;

    // Fill 2D vector with random boolean values
    let mut start:Vec<Vec<bool>> = vec![vec![false; x_usize];y_usize];
    for i in 0..y_usize {
        for j in 0..x_usize {
            start[i][j] = rand::random_bool(INIT_POPULATION);
        }
    }

    const FRAMETIME:u64 = 1000/FPS;
    // Multithread loop through frames.
    tokio::spawn(async move {
        loop {
            let mut neighbors = cells::get_all_neighbors(&start,x_usize,y_usize);
            // Clear screen before starting print.
            if execute!(stdout, terminal::Clear(terminal::ClearType::All)).is_err() {
                eprintln!("Unable to clear terminal!");
            }

            // Print by line
            for i in 0..y {
                let buf = start[i as usize].iter().map(|&v| if v {"█"} else {" "}).collect::<String>(); // Map true and false to lit up cell and whitespace.
                if execute!(stdout, cursor::MoveTo(0,i), style::PrintStyledContent(buf.blue())).is_err() {
                    eprintln!("Frame render failed!");
                }// Queue to print.
            }

            // Start calculating next and wait atleast 70ms
            neighbors = cells::calculate_next(&mut start,neighbors,y_usize,x_usize);
            tokio::time::sleep(Duration::from_millis(FRAMETIME)).await;
        }
    });

    // To ensure program doesn't exit before first iteration
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
