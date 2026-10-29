mod cells; // Contains conway's game of life next frame gen for 2d vector
mod edit_system; // Contains initial frame editor given -c flag
mod exit_function; // Contains a trigger_end function to handle all exit features.
mod render; // Contains buffered grid drawing shared by simulation and editor

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode},
    execute,
    terminal::{self, enable_raw_mode},
};
use futures::StreamExt;
use tokio::time;
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
                println!("Syntax: game-of-life [population probability|c] [framerate]");
                println!(
                    "providing c flag will run with an editor, allowing creation of first frame.\n"
                );
                println!("Keybinds:");
                println!(
                    "In editor:\n\tArrow keys - Move cursor\n\tReturn - Birth/Kill cell\n\tSpace - Begin simulation"
                );
                println!("Global:\n\tq - Quit");
                // Placeholder values to exit immidiately.
                std::process::exit(0);
            }
            init_population = DEFAULT_POPULATION;
            if population_str == "c" {
                // Editor flag
                create_flag = true;
            } else {
            eprintln!(
                "Argument '{population_str}' could not be parsed, reverting to default population."
            );
            }
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
    let frametime: f64 = 1.0 / fps as f64;
    let mut delay = time::interval(Duration::from_secs_f64(frametime));
    delay.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    // Adjustments for input ignore and clean display.
    enable_raw_mode().expect("Unable to enter raw mode.");
    // Restore the terminal before any panic message prints, so it stays readable.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        exit_function::restore_terminal(&mut io::stdout());
        default_hook(info);
    }));
    execute!(stdout, terminal::EnterAlternateScreen).expect("Unable to enter alternate screen.");
    let _ = execute!(stdout, cursor::Hide); // Cosmetic, safe to ignore

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
        let (mut width, mut height) = (x_usize, y_usize);
        let mut neighbors = cells::get_all_neighbors(&start, width, height);
        let mut frame: Vec<u8> = Vec::new();
        loop {
            // Map true and false to lit up cell and whitespace.
            if let Err(e) =
                render::draw_grid(&mut stdout, &mut frame, &start, |_, _, v| if v { "██" } else { "  " })
            {
                exit_function::trigger_exit(&mut stdout, 1, Some(&format!("Frame render failed: {e}")));
            }

            // Start calculating next and wait
            neighbors = cells::calculate_next(&mut start, neighbors, height, width);

            // Wait for the next tick; input is handled without advancing the frame.
            loop {
                tokio::select! {
                    _ = delay.tick() => break,
                    maybe_event = reader.next() => match maybe_event {
                        Some(Ok(Event::Resize(cols, rows))) => {
                            // Keep overlapping cells, pad new space with dead cells.
                            width = (cols / 2) as usize;
                            height = rows as usize;
                            start.resize(height, vec![false; width]);
                            for row in start.iter_mut() {
                                row.resize(width, false);
                            }
                            neighbors = cells::get_all_neighbors(&start, width, height);
                            let _ = execute!(stdout, terminal::Clear(terminal::ClearType::All));
                        }
                        Some(Ok(event)) => {
                            // Exit on pressing q
                            if event == Event::Key(KeyCode::Char(EXIT_CHAR).into()) {
                                exit_function::trigger_exit(&mut stdout, 0, None);
                            }
                        }
                        Some(Err(e)) => {
                            exit_function::trigger_exit(&mut stdout, 1, Some(&format!("Input error: {e}")))
                        }
                        None => exit_function::trigger_exit(&mut stdout, 0, None),
                    }
                }
            }
        }
    })
    .await
    .expect("Failed to start frame-generator!");
}
