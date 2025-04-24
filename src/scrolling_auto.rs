use crate::spreadsheet::Spreadsheet;
use crate::display::scroller_display;
use crate::input_parser::cell_parser;

/// Jump directly to a given (row, col)
pub fn scroll_to(spreadsheet: &mut Spreadsheet, row: usize, col: usize) {
    spreadsheet.curr_x = col;
    spreadsheet.curry  = row;
}

/// Handle commands, including "scroll_to A1"
pub fn scroller(cmd: &str, spreadsheet: &mut Spreadsheet) -> i32 {
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
            return 0; // success
        } else {
            return 1; // error
        }
    }

    // Fallback to scroll movement (w, a, s, d)
    scroller_display(
        cmd,
        &spreadsheet.arr,
        &mut spreadsheet.curr_x,
        &mut spreadsheet.curry,
        spreadsheet.cols,
        spreadsheet.rows,
        &mut spreadsheet.graph,
    );
    0 // assume other commands like 'w'/'a'/'s'/'d' are always valid
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spreadsheet::{initialize_spreadsheet, Spreadsheet};

    #[test]
    fn test_scroll_to() {
        let mut spreadsheet = initialize_spreadsheet(10, 10);
        scroll_to(&mut spreadsheet, 5, 5);
        assert_eq!(spreadsheet.curry, 5);
        assert_eq!(spreadsheet.curr_x, 5);
    }

    #[test]
    fn test_scroller_scroll_to() {
        let mut spreadsheet = initialize_spreadsheet(10, 10);
        scroller("scroll_to B2", &mut spreadsheet);
        assert_eq!(spreadsheet.curry, 1);
        assert_eq!(spreadsheet.curr_x, 1);
    }

    #[test]
    fn test_scroller_invalid_command() {
        let mut spreadsheet = initialize_spreadsheet(10, 10);
        scroller("invalid_command", &mut spreadsheet);
        assert_eq!(spreadsheet.curry, 0);
        assert_eq!(spreadsheet.curr_x, 0);
    }

    #[test]
    fn test_scroll_up() {
        let mut spreadsheet = initialize_spreadsheet(10, 10);
        spreadsheet.curry = 5;
        scroll_up(&mut spreadsheet);
        assert_eq!(spreadsheet.curry, 0);
    }

    #[test]
    fn test_scroll_up2() {
        let mut spreadsheet = initialize_spreadsheet(100, 100);
        spreadsheet.curry = 20;
        scroll_up(&mut spreadsheet);
        assert_eq!(spreadsheet.curry, 10);
    }

    #[test]
    fn test_scroll_down() {
        let mut spreadsheet = initialize_spreadsheet(100, 100);
        scroll_down(&mut spreadsheet);
        assert_eq!(spreadsheet.curry, 10);
    }



    #[test]
    fn test_scroll_left() {
        let mut spreadsheet = initialize_spreadsheet(10, 10);
        spreadsheet.curr_x = 5;
        scroll_left(&mut spreadsheet);
        assert_eq!(spreadsheet.curr_x, 0);
    }

    #[test]
    fn test_scroll_left2() {
        let mut spreadsheet = initialize_spreadsheet(100, 100);
        spreadsheet.curr_x = 20;
        scroll_left(&mut spreadsheet);
        assert_eq!(spreadsheet.curr_x, 10);
    }

    #[test]
    fn test_scroll_right() {
        let mut spreadsheet = initialize_spreadsheet(100, 100);
        scroll_right(&mut spreadsheet);
        assert_eq!(spreadsheet.curr_x, 10);
    }
}