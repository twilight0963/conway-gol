use std::cmp;
// Count how many neighbors a particular cell has
fn get_neighbors(grid:&Vec<Vec<bool>>, row:usize, col:usize, m:usize, n:usize) -> i32 {
    let mut count:i32=0; 

    // Check all 9 neighbors
    for i in usize::saturating_sub(row, 1)..cmp::min(m,usize::saturating_add(row, 2)) {
        for j in usize::saturating_sub(col,1)..cmp::min(n,usize::saturating_add(col,2)) {
            if i==row && j==col {
                // Don't count self
                continue;
            }

            if grid[i][j] {
                count+=1;
            }
        }
    }

    count
}

// Calculate the next frame using get_neighbors and conway's rules
pub fn calculate_next(cur:Vec<Vec<bool>>) -> Vec<Vec<bool>> {

    let m:usize = cur.len();
    let n:usize = cur[0].len();

    // Init as all false
    let mut next_frame:Vec<Vec<bool>> = vec![vec![false; n];m];

    let mut neighbors:i32;
    for row in 0..m {
        for col in 0..n {
            neighbors = get_neighbors(&cur, row, col, m, n);
            // Apply conway's rules
            if cur[row][col] {
                if neighbors < 2 || neighbors > 3 {
                    next_frame[row][col] = false;
                } else {
                    next_frame[row][col] = true;
                }
            } else {
                if neighbors == 3 {
                    next_frame[row][col] = true;
                } else {
                    next_frame[row][col] = false;
                }
            }
        }
    }

    next_frame
}

