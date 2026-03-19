mod cells; // Contains conway's game of life next frame gen for 2d vector

use std::{env::args, io};
use futures::{StreamExt, select, future::FutureExt};
use futures_timer::Delay;
use crossterm::{
    cursor, event::{Event, EventStream, KeyCode}, execute, style::{self,Stylize}, terminal::{self, enable_raw_mode}
};
use std::{time::Duration};

#[tokio::main]
async fn main() {
    const FPS:u64 = 20; // Frames per second for cell rendering
    const FRAMETIME:u64 = 1000/FPS;
    let init_population:f64;

    // Use program args for initial population chance, default = 0.1
    let mut args = args().skip(1); 
    match args.next() {
        Some(population_str) => match population_str.parse::<f64>() {
            Ok(n) => {
                // Chance must be within 0 and 1, inclusive
                if n <= 0.0 {
                    eprintln!("Population cannot be negative or zero! Defaulting to 0.1");
                    init_population = 0.1;
                } else if n >= 1.0 {
                    eprintln!("Population cannot exceed 1.0! Defaulting to 0.1");
                    init_population = 0.1;
                } else {
                    init_population = n;
                }
            },
            Err(_) => {
                // Non-f64 value given
                eprintln!("Population must be a floating integer in range 0-1! Defaulting to 0.1");
                    init_population = 0.1;
            }
        }
        // No argument given
        None => init_population = 0.1,
    }

    let mut stdout = io::stdout();

    // Adjustments for input ignore and clean display.
    if enable_raw_mode().is_err() {
    }
    if execute!(stdout,terminal::EnterAlternateScreen).is_err() {
        eprintln!("Unable to enter alternate screen");
    }
    if execute!(stdout, cursor::Hide).is_err(){
        eprintln!("Unable to hide cursor!");
    }

    // Get terminal size
    let (x,y) = terminal::size().expect("Terminal size not detected! Exiting...\n");
    let x_usize = (x/2) as usize;
    let y_usize = y as usize;

    // Fill 2D vector with random boolean values
    let mut start:Vec<Vec<bool>> = vec![vec![false; x_usize];y_usize];
    for i in 0..y_usize {
        for j in 0..x_usize {
            start[i][j] = rand::random_bool(init_population);
        }
    }

    // Multithread loop through frames.
    tokio::spawn(async move {
        loop {
            let mut neighbors = cells::get_all_neighbors(&start,x_usize,y_usize);
            // Clear screen before starting print.

            // Print by line
            for i in 0..y {
                let buf = start[i as usize].iter().map(|&v| if v {"██"} else {"  "}).collect::<String>(); // Map true and false to lit up cell and whitespace.
                if execute!(stdout, cursor::MoveTo(0,i), style::PrintStyledContent(buf.blue())).is_err() {
                    eprintln!("Frame render failed!");
                }// Queue to print.
            }

            // Start calculating next and wait
            neighbors = cells::calculate_next(&mut start,neighbors,y_usize,x_usize);

            // Check for any exit signal while waiting
            let mut reader = EventStream::new();
            let mut delay = Delay::new(Duration::from_millis(FRAMETIME)).fuse();
            let mut event = reader.next().fuse();
            select! {
                _ = delay => { continue; },
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            // Exit on pressing q
                            if event == Event::Key(KeyCode::Char('q').into()) {
                                if execute!(stdout,terminal::LeaveAlternateScreen).is_err() {
                                    eprintln!("Unable to enter alternate screen");
                                }
                                break;
                            }
                        }
                        Some(Err(e)) => eprintln!("Error: {e:?}\r"),
                        None => break,
                    }
                }
            }
        }
    }).await.expect("Failed to start frame-generator!");

}
