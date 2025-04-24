use crate::display::printer;
use crate::graph::{Formula, Graph};
use std::collections::VecDeque;
/// The core spreadsheet model: a 2D grid of `i32` cells with
/// dependency tracking and undo/redo history.
///
/// - `rows`, `cols`: dimensions of the sheet
/// - `arr`: flat row-major storage of cell values
/// - `graph`: dependency graph for formula recalculation
/// - `formula_array`: parsed formulas for each cell
/// - `formula_strings`: the original text of each cell’s formula
/// - `undo_stack` / `redo_stack`: history for undo/redo operations
/// - `curr_x`, `curry`: viewport origin for on-screen printing
/// - `output_disabled`: if `true`, suppress output on updates
///
/// # Examples
///
/// ```rust
/// # use lab1::spreadsheet::initialize_spreadsheet;
/// let mut sheet = initialize_spreadsheet(3, 4);
/// assert_eq!(sheet.rows, 3);
/// assert_eq!(sheet.cols, 4);
/// assert_eq!(sheet.arr.len(), 12);
/// assert!(sheet.formula_strings.iter().all(|s| s.is_empty()));
/// ```

pub struct Spreadsheet {
    pub rows: usize,
    pub cols: usize,
    pub arr: Vec<i32>,
    pub graph: Graph,
    pub formula_array: Vec<Formula>,
    pub output_disabled: bool,
    pub display: bool,
    pub time: f64,
    pub curr_x: usize,
    pub curry: usize,
    pub formula_strings: Vec<String>, // Store formulas as strings
    pub undo_stack: VecDeque<(usize, String)>, // Store previous formulas for undo
    pub redo_stack: VecDeque<(usize, String)>,
}

impl Spreadsheet {
    /// Print the current 10×10 viewport of this spreadsheet to stdout.
    ///
    /// This calls into the [`printer`] function with this sheet’s
    /// `curr_x`, `curry`, `arr`, `cols` and `rows`.
    ///
    /// # Panics
    ///
    /// Will panic if the underlying `printer` panics (it normally does not).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lab1::spreadsheet::initialize_spreadsheet;
    /// # let sheet = initialize_spreadsheet(2, 2);
    /// sheet.print(); // prints a 2×2 sheet
    /// ```
    pub fn print(&self) {
        printer(self.curr_x, self.curry, &self.arr, self.cols, self.rows);
    }
}
/// Create a new `Spreadsheet` of the given dimensions, initialized to all zeros.
///
/// - `rows` must be ≥ 1 and reasonably small (you control in your CLI/web UI).
/// - `cols` likewise.
///
/// # Returns
///
/// A `Spreadsheet` with:
/// - `arr` filled with `rows * cols` zeros
/// - `formula_array` filled with empty formulas (type `0`)
/// - `formula_strings` all empty
/// - empty undo/redo stacks
///
/// # Examples
///
/// ```rust
/// # use lab1::spreadsheet::initialize_spreadsheet;
/// let sheet = initialize_spreadsheet(4, 5);
/// assert_eq!(sheet.arr, vec![0; 20]);
/// assert_eq!(sheet.formula_array.len(), 20);
/// assert_eq!(sheet.formula_strings.len(), 20);
/// assert!(sheet.undo_stack.is_empty());
/// assert!(sheet.redo_stack.is_empty());
/// ```
pub fn initialize_spreadsheet(rows: usize, cols: usize) -> Spreadsheet {
    let total_cells = rows * cols;
    Spreadsheet {
        rows,
        cols,
        arr: vec![0; total_cells],
        graph: Graph::new(),
        formula_array: vec![
            Formula {
                op_type: 0,
                p1: 0,
                p2: 0,
            };
            total_cells
        ],
        output_disabled: false,
        display: true,
        time: 0.0,
        curr_x: 0,
        curry: 0,
        formula_strings: vec!["".to_string(); rows * cols],
        undo_stack: VecDeque::new(),
        redo_stack: VecDeque::new(),
    }
}
/// Convenience wrapper around [`Spreadsheet::print`].
///
/// # Examples
///
/// ```rust
/// # use lab1::spreadsheet::{initialize_spreadsheet, print_spreadsheet};
/// let sheet = initialize_spreadsheet(2, 3);
/// print_spreadsheet(&sheet);
/// ```
pub fn print_spreadsheet(spreadsheet: &Spreadsheet) {
    spreadsheet.print();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::printer;

    #[test]
    fn test_print() {
        let spreadsheet = initialize_spreadsheet(5, 5);
        // Ensure the print method runs without panicking
        spreadsheet.print();
    }

    #[test]
    fn test_print_spreadsheet() {
        let spreadsheet = initialize_spreadsheet(5, 5);
        // Ensure the print_spreadsheet function runs without panicking
        print_spreadsheet(&spreadsheet);
    }
}
