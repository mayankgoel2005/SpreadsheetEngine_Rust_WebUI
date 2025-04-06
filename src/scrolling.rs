use crate::spreadsheet::Spreadsheet;
use crate::display::scroller_display;

pub fn scroll_to(spreadsheet: &mut Spreadsheet, row: usize, col: usize) {
    spreadsheet.curr_x = col;
    spreadsheet.curry = row;
}

pub fn scroller(cmd: &str, spreadsheet: &mut Spreadsheet) {
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

pub fn scroll_up(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.curry < 10 {
        spreadsheet.curry = 0;
    } else {
        spreadsheet.curry -= 10;
    }
}

pub fn scroll_down(spreadsheet: &mut Spreadsheet) {
    let remaining = spreadsheet.rows.saturating_sub(spreadsheet.curry + 10);
    if remaining < 10 {
        spreadsheet.curry += remaining;
    } else {
        spreadsheet.curry += 10;
    }
}

pub fn scroll_left(spreadsheet: &mut Spreadsheet) {
    if spreadsheet.curr_x < 10 {
        spreadsheet.curr_x = 0;
    } else {
        spreadsheet.curr_x -= 10;
    }
}

pub fn scroll_right(spreadsheet: &mut Spreadsheet) {
    let remaining = spreadsheet.cols.saturating_sub(spreadsheet.curr_x + 10);
    if remaining < 10 {
        spreadsheet.curr_x += remaining;
    } else {
        spreadsheet.curr_x += 10;
    }
}
