mod cells; // Contains conway's game of life next frame gen for 2d vector

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode},
    execute,
    style::{self, Stylize},
    terminal::{self, enable_raw_mode},
};
use futures::{StreamExt, future::FutureExt, select};
use futures_timer::Delay;
use std::{
    env::{Args, args},
    io,
    time::Duration,
};

/// Collects all arguments from issued command and processes
/// and sanitizes
/// **Input** -> f64 population and u64 FPS in arguments
///     - Index 1 -> Chance of cell being populated as f64 parseable string
///     - Index 2 -> Frames per second as u64 parseable string
/// **Output** -> Sanitized f64 population and u64 FPS
/// **Defaults** -> Population = 0.1, FPS = 24
fn collect_args(args: &mut Args) -> (f64, u64) {
    const DEFAULT_POPULATION: f64 = 0.1;
    const DEFAULT_FPS: u64 = 24;
    let init_population: f64;
    let fps: u64;
    args.next();
    // First Arg -> Init population
    // Second Arg -> FPS
    match args.next() {
        Some(population_str) => match population_str.parse::<f64>() {
            Ok(n) => {
                // Chance must be within 0 and 1, inclusive
                if n < 1.0 && n > 0.0 {
                    init_population = n;
                } else {
                    init_population = DEFAULT_POPULATION;
                }
            }
            Err(_) => {
                // Non-f64 value given
                init_population = DEFAULT_POPULATION;
            }
        },
        // No argument given
        None => init_population = DEFAULT_POPULATION,
    };

    match args.next() {
        Some(fps_str) => match fps_str.parse::<u64>() {
            Ok(n) => {
                if n == 0 {
                    eprintln!("FPS must be a natural number!");
                    fps = DEFAULT_FPS;
                } else {
                    // Useable value given
                    fps = n;
                }
            }
            Err(_) => {
                // Unuseable value given
                eprintln!("FPS must be a natural number!");
                fps = DEFAULT_FPS;
            }
        },
        // No value given
        None => fps = DEFAULT_FPS,
    }
    (init_population, fps)
}

#[tokio::main]
async fn main() {
    const EXIT_CHAR: char = 'q';
    let init_population: f64;
    let fps: u64;

    // Use program args for initial population chance, default = 0.1
    (init_population, fps) = collect_args(&mut args());
    let mut stdout = io::stdout();
    let frametime: u64 = 1000 / fps;

    // Adjustments for input ignore and clean display.
    if enable_raw_mode().is_err() {}
    if execute!(stdout, terminal::EnterAlternateScreen).is_err() {
        eprintln!("Unable to enter alternate screen");
    }
    if execute!(stdout, cursor::Hide).is_err() {
        eprintln!("Unable to hide cursor!");
    }

    // Get terminal size
    let (x, y) = terminal::size().expect("Terminal size not detected! Exiting...\n");
    let x_usize = (x / 2) as usize;
    let y_usize = y as usize;

    // Fill 2D vector with random boolean values
    let mut start: Vec<Vec<bool>> = vec![vec![false; x_usize]; y_usize];
    for i in 0..y_usize {
        for j in 0..x_usize {
            start[i][j] = rand::random_bool(init_population);
        }
    }

    // Multithread loop through frames.
    tokio::spawn(async move {
        loop {
            let mut neighbors = cells::get_all_neighbors(&start, x_usize, y_usize);
            // Clear screen before starting print.

            // Print by line
            for i in 0..y {
                let buf = start[i as usize]
                    .iter()
                    .map(|&v| if v { "██" } else { "  " })
                    .collect::<String>(); // Map true and false to lit up cell and whitespace.
                if execute!(
                    stdout,
                    cursor::MoveTo(0, i),
                    style::PrintStyledContent(buf.blue())
                )
                .is_err()
                {
                    eprintln!("Frame render failed!");
                } // Queue to print.
            }

            // Start calculating next and wait
            neighbors = cells::calculate_next(&mut start, neighbors, y_usize, x_usize);

            // Check for any exit signal while waiting
            let mut reader = EventStream::new();
            let mut delay = Delay::new(Duration::from_millis(frametime)).fuse();
            let mut event = reader.next().fuse();
            select! {
                _ = delay => { continue; },
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            // Exit on pressing q
                            if event == Event::Key(KeyCode::Char(EXIT_CHAR).into()) {
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
    })
    .await
    .expect("Failed to start frame-generator!");
}
