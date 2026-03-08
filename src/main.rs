mod cells; // Contains conway's game of life next frame gen for 2d vector

use std::thread;
use std::time::Duration;


fn main() {
    // DEBUG: Start with 5x5 vector with an oscillator
    // print oscillator frames using os and whitespaces for now
    let mut start:Vec<Vec<bool>> = vec![vec![false;5];5];
    start[0][3] = true;
    start[1][3] = true;
    start[2][3] = true;

    loop {
        for i in 0..5 {
            for j in 0..5 {
                if start[i][j] {
                    print!("o");
                } else {
                    print!(" ");
                }
            }
            print!("\n");
        }
        print!("\n");

        start = cells::calculate_next(start);
        thread::sleep(Duration::from_millis(500));
    }
}
