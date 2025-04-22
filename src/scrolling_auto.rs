use crate::spreadsheet::Spreadsheet;
use crate::display::scroller_display;
use crate::input_parser::cell_parser;

/// Jump directly to a given (row, col)
pub fn scroll_to(spreadsheet: &mut Spreadsheet, row: usize, col: usize) {
    spreadsheet.curr_x = col;
    spreadsheet.curry  = row;
}

/// Handle commands, including "scroll_to A1"
pub fn scroller(cmd: &str, spreadsheet: &mut Spreadsheet) {
    // If command is of form "scroll_to CELL"
    if let Some(arg) = cmd.strip_prefix("scroll_to ") {
        let cell = arg.trim();
        // Parse cell reference (e.g., "A1") into index
        let idx = cell_parser(
            cell,
            spreadsheet.cols as i32,
            spreadsheet.rows as i32,
        );
        if idx >= 0 {
            let idx = idx as usize;
            let row = idx / spreadsheet.cols;
            let col = idx % spreadsheet.cols;
            scroll_to(spreadsheet, row, col);
        }
        return;
    }

    // Otherwise, fallback to existing scroll logic
    scroller_display(
        cmd,
        &spreadsheet.arr,
        &mut spreadsheet.curr_x,
        &mut spreadsheet.curry,
        spreadsheet.cols,
        spreadsheet.rows,
        &mut spreadsheet.graph,
    );
}

/// Scroll up by a page (10 rows)
pub fn scroll_up(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.curry < 10 {
        spreadsheet.curry = 0;
    } else {
        spreadsheet.curry -= 10;
    }
}

/// Scroll down by a page (10 rows)
pub fn scroll_down(spreadsheet: &mut Spreadsheet) {
    let remaining = spreadsheet.rows.saturating_sub(spreadsheet.curry + 10);
    spreadsheet.curry += remaining.min(10);
}

/// Scroll left by a page (10 columns)
pub fn scroll_left(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.curr_x < 10 {
        spreadsheet.curr_x = 0;
    } else {
        spreadsheet.curr_x -= 10;
    }
}

/// Scroll right by a page (10 columns)
pub fn scroll_right(spreadsheet: &mut Spreadsheet) {
    let remaining = spreadsheet.cols.saturating_sub(spreadsheet.curr_x + 10);
    spreadsheet.curr_x += remaining.min(10);
}
