use super::grid::{Pos};
use std::ops::{Index,IndexMut};

/* An infinite 2-dimensional grid where every position has default value until
 * it is set otherwise. */
#[derive(Clone)]
pub struct InfiniteGrid<T: Clone+PartialEq> {
    rows: Vec<Vec<T>>,
    default: T
}

impl<T: Clone+PartialEq> InfiniteGrid<T> {
    pub fn new(default: T) -> InfiniteGrid<T> {
        InfiniteGrid {
            rows: Vec::new(),
            default: default
        }
    }

    // Output a finite, cropped grid in the form of a Vec of Vec. Crops the
    // grid by ignoring any values that are still at the default. Contents are
    // cloned. The result may actually be an empty Vec if there are no
    // non-default values.
    pub fn crop(&self) -> Vec<Vec<T>> {
        // I'm sure this could be prettier >_>

        // Find bounds
        let mut left = None;
        let mut top = None;
        let mut right = None;
        let mut bottom = None;
        for y_index in 0..self.rows.len() {
            let row = self.rows.get(y_index).unwrap();
            let mut first_pos = None;
            let mut last_pos = None;
            for x_index in 0..row.len() {
                if row[x_index] != self.default {
                    let x_pos = Self::index_to_pos(x_index);
                    if first_pos.is_none() || x_pos < first_pos.unwrap() {
                        first_pos = Some(x_pos);
                    }
                    if last_pos.is_none() || x_pos > last_pos.unwrap() {
                        last_pos = Some(x_pos);
                    }
                }
            }
            if first_pos.is_some() {
                let y_pos = Self::index_to_pos(y_index);
                if top.is_none() || y_pos < top.unwrap() {
                    top = Some(y_pos);
                }
                if bottom.is_none() || y_pos > bottom.unwrap() {
                    bottom = Some(y_pos);
                }
                if left.is_none() || first_pos.unwrap() < left.unwrap() {
                    left = first_pos;
                }
                if right.is_none() || last_pos.unwrap() > right.unwrap() {
                    right = last_pos;
                }
            }
        }

        // Create cropped grid
        let mut result: Vec<Vec<T>> = Vec::new();
        if left.is_some() {
            let left = left.unwrap();
            let top = top.unwrap();
            let right = right.unwrap();
            let bottom = bottom.unwrap();
            for y_pos in top..=bottom {
                let y_index = Self::pos_to_index(y_pos);
                let orig_row = self.rows.get(y_index).unwrap();
                let mut row: Vec<T> = Vec::new();
                for x_pos in left..=right {
                    // Not all rows are the same width internally
                    let x_index = Self::pos_to_index(x_pos);
                    if x_index >= orig_row.len() {
                        row.push(self.default.clone());
                    } else {
                        row.push(orig_row.get(x_index).unwrap().clone());
                    }
                }
                result.push(row);
            }
        }
        result
    }

    // Ensure the underlying vectors have min_rows and min_cols capacity.
    fn ensure_capacity(&mut self, min_rows: usize, min_cols: usize) {
        let orig_num_rows = self.rows.len();
        if orig_num_rows < min_rows {
            self.rows.reserve(min_rows - orig_num_rows);
            for _ in orig_num_rows..min_rows {
                self.rows.push(Vec::with_capacity(min_cols));
            }
        }
        for row in 0..min_rows {
            if self.rows[row].len() < min_cols {
                self.rows[row].resize(min_cols, self.default.clone());
            }
        }
    }

    // Map a position (in one dimension) from the infinite (isize range) space
    // to a natural number index (usize) for the underlying Vec.
    fn pos_to_index(pos: isize) -> usize {
        let result = if pos < 0 {
                         (-pos*2) - 1
                     } else {
                         pos * 2
                     };
        result as usize
    }

    fn index_to_pos(index: usize) -> isize {
        let index = index as isize;
        if index % 2 == 0 {
            index / 2
        } else {
            -index/2 - 1
        }
    }
}

// Index into the grid at the position (row, col).
impl<T: Clone+PartialEq> Index<Pos> for InfiniteGrid<T> {
    type Output = T;

    fn index<'a>(&'a self, pos: Pos) -> &'a T {
        let row_index = Self::pos_to_index(pos.row);
        let col_index = Self::pos_to_index(pos.col);
        if self.rows.len() <= row_index ||
           self.rows[row_index].len() <= col_index {
            return &self.default
        }
        &self.rows[row_index][col_index]
    }
}

// Write into the grid at the position (row, col).
impl<T: Clone+PartialEq> IndexMut<Pos> for InfiniteGrid<T> {
    fn index_mut<'a>(&'a mut self, pos: Pos) -> &'a mut T {
        let row_index = Self::pos_to_index(pos.row);
        let col_index = Self::pos_to_index(pos.col);
        self.ensure_capacity(row_index+1, col_index+1);
        &mut self.rows[row_index][col_index]
    }
}

#[cfg(test)]
mod tests {
    use super::InfiniteGrid;
    use super::super::grid::Pos;
    use crate::pos;

    #[test]
    fn can_be_constructed_with_int() {
        let grid: InfiniteGrid<isize> = InfiniteGrid::new(0);
        assert!(grid.default == 0);
    }

    #[test]
    fn returns_default_for_out_of_bounds() {
        let grid: InfiniteGrid<isize> = InfiniteGrid::new(5);
        assert!(grid[pos!(-1000, 545)] == 5);
    }

    #[test]
    fn set_values_and_read_them_back() {
        let mut grid: InfiniteGrid<char> = InfiniteGrid::new('x');
        grid[pos!(10, 10)]  = 'a';
        grid[pos!(-5, -9)]  = 'b';
        grid[pos!(0, 0)]    = 'c';
        grid[pos!(14, -14)] = 'd';
        grid[pos!(-14, 14)] = 'e';

        assert!(grid[pos!(1, 1)]    == 'x');
        assert!(grid[pos!(10, 10)]  == 'a');
        assert!(grid[pos!(-5, -9)]  == 'b');
        assert!(grid[pos!(0, 0)]    == 'c');
        assert!(grid[pos!(14, -14)] == 'd');
        assert!(grid[pos!(-14, 14)] == 'e');
        
        // overwrite
        grid[pos!(0, 0)] = 'z';
        assert!(grid[pos!(0, 0)] == 'z');
    }
}
