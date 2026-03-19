mod cells; // Contains conway's game of life next frame gen for 2d vector

use std::io;
use futures::{StreamExt, select, future::FutureExt};
use futures_timer::Delay;
use crossterm::{
    cursor, event::{Event, EventStream, KeyCode}, execute, style::{self,Stylize}, terminal::{self, enable_raw_mode}
};
use std::{time::Duration};

#[tokio::main]
async fn main() {
    const FPS:u64 = 24; // Frames per second for cell rendering.
    const INIT_POPULATION:f64 = 0.1; // Starting chance of a cell being active.
    let mut stdout = io::stdout();
    // Try to hide cursor
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
    let x_usize = x as usize;
    let y_usize = y as usize;

    // Fill 2D vector with random boolean values
    let mut start:Vec<Vec<bool>> = vec![vec![false; x_usize];y_usize];
    for i in 0..y_usize {
        for j in 0..x_usize {
            start[i][j] = rand::random_bool(INIT_POPULATION);
        }
    }

    tokio::spawn(async move {
    });

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
            let mut reader = EventStream::new();
            let mut delay = Delay::new(Duration::from_millis(FRAMETIME)).fuse();
            let mut event = reader.next().fuse();
            select! {
                _ = delay => { continue; },
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
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
