use std::cmp;
// Count how many neighbors a particular cell has
pub fn get_neighbors(grid:&Vec<Vec<bool>>, row:usize, col:usize, m:usize, n:usize) -> i32 {
    let mut count:i32=0; 

    // Check all 9 neighbors
    for i in usize::saturating_sub(row, 1)..cmp::min(n,usize::saturating_add(row, 2)) {
        for j in usize::saturating_sub(col,1)..cmp::min(m,usize::saturating_add(col,2)) {
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

pub fn get_all_neighbors(grid:&Vec<Vec<bool>>,m:usize,n:usize) -> Vec<Vec<i32>> {
    let mut neighbors = vec![vec![0;m];n];
    for row in 0..n {
        for col in 0..m {
            neighbors[row][col] = get_neighbors(grid,row,col,m,n);
        }
    }
    neighbors
}

fn birth_cell(row:usize, col:usize, cur:&mut Vec<Vec<bool>>, neighbors:&mut Vec<Vec<i32>>, m:usize,n:usize) {
    // Mark cell as born and update neighbors
    for i in usize::saturating_sub(row, 1)..cmp::min(m,usize::saturating_add(row, 2)) {
        for j in usize::saturating_sub(col,1)..cmp::min(n,usize::saturating_add(col,2)) {
            if i==row && j==col {
                cur[row][col] = true;
                continue;
            }

            neighbors[i][j]+=1;
        }
    }
}
fn kill_cell(row:usize, col:usize, cur:&mut Vec<Vec<bool>>, neighbors:&mut Vec<Vec<i32>>, m:usize,n:usize) {
    // Mark cell as dead and update neighbors
    for i in usize::saturating_sub(row, 1)..cmp::min(m,usize::saturating_add(row, 2)) {
        for j in usize::saturating_sub(col,1)..cmp::min(n,usize::saturating_add(col,2)) {
            if i==row && j==col {
                cur[row][col] = false;
                continue;
            }

            neighbors[i][j]-=1;
        }
    }
}



// Calculate the next frame using get_neighbors and conway's rules
pub fn calculate_next(cur:&mut Vec<Vec<bool>>, neighbors:Vec<Vec<i32>>,m:usize,n:usize) -> Vec<Vec<i32>>{
    let mut new_neighbor = neighbors.clone();
    for row in 0..m {
        for col in 0..n {
            // Apply conway's rules
            if cur[row][col] {
                if neighbors[row][col] < 2 || neighbors[row][col] > 3 {
                    kill_cell(row,col,cur,&mut new_neighbor,m,n);
                }
            } else {
                if neighbors[row][col] == 3 {
                    birth_cell(row,col,cur,&mut new_neighbor,m,n);
                }
            }
        }
    }
    new_neighbor
}

