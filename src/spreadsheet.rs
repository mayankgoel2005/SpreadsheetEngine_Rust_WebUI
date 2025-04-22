use crate::graph::{Graph, Formula};
use crate::display::printer;
use std::collections::VecDeque;


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
    pub formula_strings: Vec<String>,    // Store formulas as strings
    pub undo_stack: VecDeque<(usize, String)>, // Store previous formulas for undo
    pub redo_stack: VecDeque<(usize, String)>,
}

impl Spreadsheet {
    pub fn print(&self) {
        printer(self.curr_x, self.curry, &self.arr, self.cols, self.rows);
    }
}

pub fn initialize_spreadsheet(rows: usize, cols: usize) -> Spreadsheet {
    let total_cells = rows * cols;
    Spreadsheet {
        rows,
        cols,
        arr: vec![0; total_cells],
        graph: Graph::new(rows*cols),
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

pub fn print_spreadsheet(spreadsheet: &Spreadsheet) {
    spreadsheet.print();
}
