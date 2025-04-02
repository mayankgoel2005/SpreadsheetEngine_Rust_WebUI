use crate::spreadsheet::Spreadsheet;

pub fn scroll_up(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.start_row > 0 {
        spreadsheet.start_row = spreadsheet.start_row.saturating_sub(10);
    }
}

pub fn scroll_down(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.start_row + 10 < spreadsheet.rows {
        spreadsheet.start_row = (spreadsheet.start_row + 10).min(spreadsheet.rows - 10);
    }
}

pub fn scroll_left(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.start_col > 0 {
        spreadsheet.start_col = spreadsheet.start_col.saturating_sub(10);
    }
}

pub fn scroll_right(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.start_col + 10 < spreadsheet.cols {
        spreadsheet.start_col = (spreadsheet.start_col + 10).min(spreadsheet.cols - 10);
    }
}

pub fn scroll_to(spreadsheet: &mut Spreadsheet, row: usize, col: usize) {
    spreadsheet.start_row = row.saturating_sub(1);
    spreadsheet.start_col = col.saturating_sub(1);
}
