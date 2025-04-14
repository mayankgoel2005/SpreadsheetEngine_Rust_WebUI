use std::i32;
use std::thread;
use std::time::Duration;
use crate::graph::{Graph, Formula, add_formula, add_edge};
use crate::input_parser::{cell_parser};

fn validate_range(range_start: i32, range_end: i32, cols: i32) -> bool {
    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;
    !(start_row > end_row || (start_row == end_row && start_col > end_col))
}

pub fn min_func(a: &str, cols: i32, rows: i32, pos_equal_to: usize, _pos_end: usize, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let ref_sub=&a[..pos_equal_to];
    let first_cell = cell_parser(ref_sub, cols, rows, 0, (pos_equal_to - 1) as i32, graph);
    if first_cell == -1 {
        println!("Invalid destination cell\n");
        return;
    }
    let open_paren = a[pos_equal_to..].find('(');
    let close_paren = a[pos_equal_to..].find(')');
    let colon;
    let open;
    let close;
    if let (Some(open_rel), Some(close_rel)) = (open_paren, close_paren) {
        open = pos_equal_to + open_rel;
        close = pos_equal_to + close_rel;

        if close <= open + 1 {
            println!("Invalid range: Missing or misplaced parentheses\n");
            return;
        }

        let colon_pos = a[open + 1..close].find(':');
        if colon_pos.is_none() {
            println!("Invalid range: Missing ':'\n");
            return;
        }

        colon = open + 1 + colon_pos.unwrap();
    } else {
        println!("Invalid range: Missing or misplaced parentheses\n");
        return;
    }
    let ref_sub1 = &a[open+1..colon];
    let ref_sub2=&a[colon+1..close];
    let range_start = cell_parser(ref_sub1, cols, rows, (open as i32) + 1, (colon as i32) - 1, graph);
    let range_end = cell_parser(ref_sub2, cols, rows, (colon as i32) + 1, (close as i32) - 1, graph);
    if range_start == -1 || range_end == -1 || !validate_range(range_start, range_end, cols) {
        println!("Invalid range\n");
        return;
    }
    add_formula(graph, first_cell, range_start, range_end, 9, formula_array);

    let mut min_value = arr[range_start as usize];

    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;

    if start_row == end_row {
        for idx in range_start..=range_end {
            graph.adj[idx as usize] = Some(add_edge(first_cell, graph.adj[idx as usize].take()));
            if (arr[idx as usize] < min_value) {
                min_value = arr[idx as usize];
            }
        }
    }
    else {
        for row in start_row..=end_row {
            let col_start = start_col;
            let col_end = end_col;

            for col in col_start..=col_end {
                let idx = row * cols + col;
                graph.adj[idx as usize] = Some(add_edge(first_cell, graph.adj[idx as usize].take()));
                if (arr[idx as usize] < min_value) {
                    min_value = arr[idx as usize];
                }
            }
        }
    }
    arr[first_cell as usize] = min_value;
}

pub fn max_func(a: &str, cols: i32, rows: i32, pos_equal_to: usize, _pos_end: usize, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let ref_sub=&a[..pos_equal_to];
    let first_cell = cell_parser(ref_sub, cols, rows, 0, (pos_equal_to - 1) as i32, graph);
    if first_cell == -1 {
        println!("Invalid destination cell\n");
        return;
    }
    let open_paren = a[pos_equal_to..].find('(');
    let close_paren = a[pos_equal_to..].find(')');
    let colon;
    let open;
    let close;
    if let (Some(open_rel), Some(close_rel)) = (open_paren, close_paren) {
        open = pos_equal_to + open_rel;
        close = pos_equal_to + close_rel;

        if close <= open + 1 {
            println!("Invalid range: Missing or misplaced parentheses\n");
            return;
        }

        let colon_pos = a[open + 1..close].find(':');
        if colon_pos.is_none() {
            println!("Invalid range: Missing ':'\n");
            return;
        }

        colon = open + 1 + colon_pos.unwrap();
    } else {
        println!("Invalid range: Missing or misplaced parentheses\n");
        return;
    }
    let ref_sub1 = &a[open+1..colon];
    let ref_sub2=&a[colon+1..close];
    let range_start = cell_parser(ref_sub1, cols, rows, (open as i32) + 1, (colon as i32) - 1, graph);
    let range_end = cell_parser(ref_sub2, cols, rows, (colon as i32) + 1, (close as i32) - 1, graph);
    if range_start == -1 || range_end == -1 || !validate_range(range_start, range_end, cols) {
        println!("Invalid range\n");
        return;
    }
    add_formula(graph, first_cell, range_start, range_end, 10, formula_array);

    let mut max_value = arr[range_start as usize];

    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;

    if start_row == end_row {
        for idx in range_start..=range_end {
            graph.adj[idx as usize] = Some(add_edge(first_cell, graph.adj[idx as usize].take()));
            if (arr[idx as usize] > max_value) {
                max_value = arr[idx as usize];
            }
        }
    }
    else {
        for row in start_row..=end_row {
            let col_start = start_col;
            let col_end = end_col;

            for col in col_start..=col_end {
                let idx = row * cols + col;
                graph.adj[idx as usize] = Some(add_edge(first_cell, graph.adj[idx as usize].take()));
                if arr[idx as usize] > max_value {
                    max_value = arr[idx as usize];
                }
            }
        }
    }
    println!("Max value: {}", max_value);
    arr[first_cell as usize] = max_value;
}

pub fn avg_func(a: &str, cols: i32, rows: i32, pos_equal_to: usize, _pos_end: usize, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let ref_sub=&a[..pos_equal_to];
    let first_cell = cell_parser(ref_sub, cols, rows, 0, (pos_equal_to - 1) as i32, graph);
    if first_cell == -1 {
        println!("Invalid destination cell\n");
        return;
    }
    let open_paren = a[pos_equal_to..].find('(');
    let close_paren = a[pos_equal_to..].find(')');
    let colon;
    let open;
    let close;
    if let (Some(open_rel), Some(close_rel)) = (open_paren, close_paren) {
        open = pos_equal_to + open_rel;
        close = pos_equal_to + close_rel;

        if close <= open + 1 {
            println!("Invalid range: Missing or misplaced parentheses\n");
            return;
        }

        let colon_pos = a[open + 1..close].find(':');
        if colon_pos.is_none() {
            println!("Invalid range: Missing ':'\n");
            return;
        }

        colon = open + 1 + colon_pos.unwrap();
    } else {
        println!("Invalid range: Missing or misplaced parentheses\n");
        return;
    }
    let ref_sub1 = &a[open+1..colon];
    let ref_sub2=&a[colon+1..close];
    let range_start = cell_parser(ref_sub1, cols, rows, (open as i32) + 1, (colon as i32) - 1, graph);
    let range_end = cell_parser(ref_sub2, cols, rows, (colon as i32) + 1, (close as i32) - 1, graph);
    if range_start == -1 || range_end == -1 || !validate_range(range_start, range_end, cols) {
        println!("Invalid range\n");
        return;
    }
    add_formula(graph, first_cell, range_start, range_end,  11, formula_array);

    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;
    let mut sum = 0;
    let mut count = 0;
    for row in start_row..=end_row {
        let col_start = start_col;
        let col_end = end_col;

        for col in col_start..=col_end {
            let idx = row * cols + col;
            graph.adj[idx as usize] = Some(add_edge(first_cell, graph.adj[idx as usize].take()));
            sum += arr[idx as usize];
            count += 1;
        }
    }
    let avg_value = if count == 0 { 0 } else { sum / count };
    arr[first_cell as usize] = avg_value;
}

pub fn sum_func(a: &str, cols: i32, rows: i32, pos_equal_to: usize, _pos_end: usize, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let ref_sub=&a[..pos_equal_to];
    let first_cell = cell_parser(ref_sub, cols, rows, 0, (pos_equal_to - 1) as i32, graph);
    if first_cell == -1 {
        println!("Invalid destination cell\n");
        return;
    }
    let open_paren = a[pos_equal_to..].find('(');
    let close_paren = a[pos_equal_to..].find(')');
    let colon;
    let open;
    let close;
    if let (Some(open_rel), Some(close_rel)) = (open_paren, close_paren) {
        open = pos_equal_to + open_rel;
        close = pos_equal_to + close_rel;

        if close <= open + 1 {
            println!("Invalid range: Missing or misplaced parentheses\n");
            return;
        }

        let colon_pos = a[open + 1..close].find(':');
        if colon_pos.is_none() {
            println!("Invalid range: Missing ':'\n");
            return;
        }

        colon = open + 1 + colon_pos.unwrap();
    } else {
        println!("Invalid range: Missing or misplaced parentheses\n");
        return;
    }
    let ref_sub1 = &a[open+1..colon];
    let ref_sub2=&a[colon+1..close];
    let range_start = cell_parser(ref_sub1, cols, rows, (open as i32) + 1, (colon as i32) - 1, graph);
    let range_end = cell_parser(ref_sub2, cols, rows, (colon as i32) + 1, (close as i32) - 1, graph);
    if range_start == -1 || range_end == -1 || !validate_range(range_start, range_end, cols) {
        println!("Invalid range\n");
        return;
    }
    add_formula(graph, first_cell, range_start, range_end, 12, formula_array);

    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;
    let mut sum = 0;
    for row in start_row..=end_row {
        let col_start = start_col;
        let col_end = end_col;

        for col in col_start..=col_end {
            let idx = row * cols + col;
            graph.adj[idx as usize] = Some(add_edge(first_cell, graph.adj[idx as usize].take()));
            sum += arr[idx as usize];
        }
    }
    arr[first_cell as usize] = sum;
}

pub fn standard_dev_func(a: &str, cols: i32, rows: i32, pos_equal_to: usize, _pos_end: usize, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let ref_sub=&a[..pos_equal_to];
    let first_cell = cell_parser(ref_sub, cols, rows, 0, (pos_equal_to - 1) as i32, graph);
    if first_cell == -1 {
        println!("Invalid destination cell\n");
        return;
    }
    let open_paren = a[pos_equal_to..].find('(');
    let close_paren = a[pos_equal_to..].find(')');
    let colon;
    let open;
    let close;
    if let (Some(open_rel), Some(close_rel)) = (open_paren, close_paren) {
        open = pos_equal_to + open_rel;
        close = pos_equal_to + close_rel;

        if close <= open + 1 {
            println!("Invalid range: Missing or misplaced parentheses\n");
            return;
        }

        let colon_pos = a[open + 1..close].find(':');
        if colon_pos.is_none() {
            println!("Invalid range: Missing ':'\n");
            return;
        }

        colon = open + 1 + colon_pos.unwrap();
    } else {
        println!("Invalid range: Missing or misplaced parentheses\n");
        return;
    }
    let ref_sub1 = &a[open+1..colon];
    let ref_sub2=&a[colon+1..close];
    let range_start = cell_parser(ref_sub1, cols, rows, (open as i32) + 1, (colon as i32) - 1, graph);
    let range_end = cell_parser(ref_sub2, cols, rows, (colon as i32) + 1, (close as i32) - 1, graph);
    if range_start == -1 || range_end == -1 || !validate_range(range_start, range_end, cols) {
        println!("Invalid range\n");
        return;
    }
    add_formula(graph, first_cell, range_start, range_end, 13, formula_array);

    let start_row = range_start / cols;
    let start_col = range_start % cols;
    let end_row = range_end / cols;
    let end_col = range_end % cols;
    let mut sum = 0;
    let mut count = 0;
    for row in start_row..=end_row {
        let col_start = start_col;
        let col_end = end_col;

        for col in col_start..=col_end {
            let idx = row * cols + col;
            graph.adj[idx as usize] = Some(add_edge(first_cell, graph.adj[idx as usize].take()));
            sum += arr[idx as usize];
            count += 1;
        }
    }
    let avg_value = if count == 0 { 0 } else { sum / count };
    let mut standard_dev_squared = 0;
    for i in start_row..=end_row {
        let col_start = start_col;
        let col_end = end_col;

        for j in col_start..=col_end {
            let idx = i * cols + j;
            let prod = (arr[idx as usize] - avg_value) * (arr[idx as usize] - avg_value);
            standard_dev_squared += prod;
        }
    }
    let standard_dev = ((standard_dev_squared / count) as f64).sqrt() as i32;
    arr[first_cell as usize] = standard_dev;
}

pub fn sleep_func(a: &str, cols: i32, rows: i32, pos_equal_to: usize, _pos_end: usize, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let target_cell = cell_parser(a, cols, rows, 0, (pos_equal_to - 1) as i32, graph);
    if target_cell == -1 {
        println!("Invalid destination cell\n");
        return;
    }
    let open_paren = a[pos_equal_to..].find('(');
    let close_paren = a[pos_equal_to..].find(')');
    if let (Some(&open_paren), Some(&close_paren)) = (open_paren.as_ref(), close_paren.as_ref()) {
        if close_paren <= open_paren + 1 {
            println!("Invalid range: Missing or misplaced parentheses\n");
            return;
        }
    }else{
        println!("Invalid range: Missing or misplaced parentheses\n");
        return;
    }
    let open = open_paren.unwrap();
    let close = close_paren.unwrap();
    let ref_cell = cell_parser(a, cols, rows, (open as i32) +1, (close as i32)-1, graph);
    let sleep_value: i32;

    if ref_cell != -1 {
        sleep_value = arr[ref_cell as usize];

        if sleep_value == i32::MIN {
            println!("Referenced cell {} contains an error value", ref_cell);
            arr[target_cell as usize] = i32::MIN;
            return;
        }

        graph.adj[ref_cell as usize] = Some(add_edge(target_cell, graph.adj[ref_cell as usize].take()));
    } else {
        match a[open + 1..close].trim().parse::<i32>() {
            Ok(val) => sleep_value = val,
            Err(_) => {
                println!("Invalid sleep value");
                arr[target_cell as usize] = i32::MIN;
                return;
            }
        }

        if sleep_value <= 0 {
            arr[target_cell as usize] = sleep_value;
            add_formula(graph, target_cell, if ref_cell != -1 { ref_cell } else { target_cell }, sleep_value, 14, formula_array,);
            return;
        }
    }

    if sleep_value <= 0 {
        arr[target_cell as usize] = sleep_value;
        add_formula(graph, target_cell, if ref_cell != - 1 { ref_cell } else { target_cell }, sleep_value,14, formula_array,);
        return;
    }

    add_formula(graph, target_cell, if ref_cell != -1 { ref_cell } else { target_cell }, sleep_value, 14, formula_array,);
    arr[target_cell as usize] = sleep_value;
}