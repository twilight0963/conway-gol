mod cells; // Contains conway's game of life next frame gen for 2d vector
mod edit_system; // Contains initial frame editor given -c flag

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
/// - **Input** -> f64 population and u64 FPS in arguments
///   1. -> Chance of cell being populated as f64 parseable string or c or --help or -h
///   2. -> Frames per second as u64 parseable string
/// - **Output** -> Sanitized f64 population and u64 FPS
/// - **Defaults** -> Population = 0.1, FPS = 24
fn collect_args(args: &mut Args) -> (f64, u64, bool) {
    const DEFAULT_POPULATION: f64 = 0.1;
    const DEFAULT_FPS: u64 = 24;
    let init_population: f64;
    let fps: u64;
    let mut create_flag: bool = false;
    args.next();
    // First Arg -> Init population
    // Second Arg -> FPS
    if let Some(population_str) = args.next() {
        if let Ok(n) = population_str.parse::<f64>() {
            // Chance must be within 0 and 1, inclusive
            if n <= 1.0 && n > 0.0 {
                init_population = n;
            } else {
                init_population = DEFAULT_POPULATION;
            }
        } else {
            if population_str == "--help" || population_str == "-h" {
                // Help function
                println!("Syntax: game-of-life [population probability|c] [framerate]\n");
                println!(
                    "providing c flag will run with an editor, allowing creation of first frame.\n\n"
                );
                println!("Keybinds:");
                println!(
                    "In editor:\n\tArrow keys - Move cursor\n\tReturn - Birth/Kill cell\n\tSpace - Begin simulation"
                );
                println!("In simulation:\n\tq - Exit simulation");
                // Placeholder values to exit immidiately.
                std::process::exit(0);
            }
            if population_str == "c" {
                // Editor flag
                create_flag = true;
            }
            init_population = DEFAULT_POPULATION;
        }
    } else {
        init_population = DEFAULT_POPULATION
    };

    if let Some(fps_str) = args.next() {
        if let Ok(n) = fps_str.parse::<u64>() {
            if n == 0 {
                eprintln!("FPS must be a natural number!");
                fps = DEFAULT_FPS;
            } else {
                // Useable value given
                fps = n;
            }
        } else {
            // Unuseable value given
            eprintln!("FPS must be a natural number!");
            fps = DEFAULT_FPS;
        }
    } else {
        fps = DEFAULT_FPS
    }

    (init_population, fps, create_flag)
}

#[tokio::main]
async fn main() {
    const EXIT_CHAR: char = 'q';
    let init_population: f64;
    let fps: u64;
    let create_flag: bool;

    // Use program args for initial population chance, default = 0.1
    (init_population, fps, create_flag) = collect_args(&mut args());
    let mut stdout = io::stdout();
    let frametime: u64 = 1000 / fps;

    // Adjustments for input ignore and clean display.
    enable_raw_mode().expect("Unable to enter raw mode.");
    execute!(stdout, terminal::EnterAlternateScreen).expect("Unable to enter alternate screen.");
    if execute!(stdout, cursor::Hide).is_err() {
        eprintln!("Unable to hide cursor!");
    }

    // Get terminal size
    let (x, y) = terminal::size().expect("Terminal size not detected! Exiting...\n");
    let x_usize = (x / 2) as usize;
    let y_usize = y as usize;

    // Fill 2D vector with random boolean values
    let mut start: Vec<Vec<bool>> = vec![vec![false; x_usize]; y_usize];

    // Check for c flag before filling start vec
    let mut reader = EventStream::new();
    if create_flag {
        edit_system::edit_frame(&mut start, x, y, &mut reader, &mut stdout)
            .await
            .expect("Unable to start editor");
    } else {
        for i in 0..y_usize {
            for j in 0..x_usize {
                start[i][j] = rand::random_bool(init_population);
            }
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
