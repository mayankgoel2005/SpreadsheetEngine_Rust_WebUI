use crate::display::printer;
use crate::graph::{Formula, Graph};

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
}

impl Spreadsheet {
    pub fn print(&self) {
        printer(self.curr_x, self.curry, &self.arr, self.cols, self.rows);
    }
}

/// Construct a new `Spreadsheet` with the given dimensions.
///
/// All cells are initialized to zero and no formulas are set.
///
/// # Panics
///
/// Will panic if `rows * cols` overflows `usize` (unlikely for sane sizes).
///
/// # Examples
///
/// ```rust
/// use lab1::initialize_spreadsheet;
///
/// let sheet = initialize_spreadsheet(5, 10);
/// assert_eq!(sheet.rows, 5);
/// assert_eq!(sheet.cols, 10);
/// assert_eq!(sheet.arr.len(), 5 * 10);
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
    }
}

pub fn print_spreadsheet(spreadsheet: &Spreadsheet) {
    spreadsheet.print();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_spreadsheet() {
        let rows = 5;
        let cols = 10;
        let spreadsheet = initialize_spreadsheet(rows, cols);

        assert_eq!(spreadsheet.rows, rows);
        assert_eq!(spreadsheet.cols, cols);
        assert_eq!(spreadsheet.arr.len(), rows * cols);
        assert!(spreadsheet.arr.iter().all(|&x| x == 0)); // All cells initialized to 0
        assert_eq!(spreadsheet.curr_x, 0);
        assert_eq!(spreadsheet.curry, 0);
        assert!(spreadsheet.formula_array.iter().all(|f| f.op_type == 0)); // All formulas initialized
    }

    #[test]
    fn test_print_spreadsheet() {
        let rows = 5;
        let cols = 5;
        let spreadsheet = initialize_spreadsheet(rows, cols);

        // Ensure the print_spreadsheet function runs without panicking
        print_spreadsheet(&spreadsheet);
    }
}