use crate::spreadsheet::Spreadsheet;
use crate::graph::{
    Graph, Formula, arith, add_formula, add_edge, delete_edge, recalculate, add_edge_formula
};
use crate::functions::{
    max_func, sum_func, standard_dev_func, avg_func, min_func, sleep_func
};

// Keep track of old values in case you need to revert them.
static mut OLD_VALUE: i32 = 0;
static mut OLD_OP_TYPE: i32 = 0;
static mut OLD_P1: i32 = 0;
static mut OLD_P2: i32 = 0;
pub static mut HAS: i32 = 0;

// Utility: Check if char is A-Z
fn is_alpha(c: char) -> bool {
    c >= 'A' && c <= 'Z'
}

// Utility: Check if char is 0-9
fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

/// Parse a cell reference like "A1", returning a 1D index (row * cols + col)
/// or -1 if invalid/out of range.
pub fn cell_parser(
    formula: &str,
    cols: i32,
    rows: i32,
    _start: i32,
    _end: i32,
    _graph: &mut Graph
) -> i32 {
    let mut cell_col = 0;
    let mut cell_row = 0;
    let mut dfound = false;

    for ch in formula.chars() {
        if is_alpha(ch) {
            if !dfound {
                // Convert letter(s) to 1-based col index
                cell_col = cell_col * 26 + (ch as i32 - 'A' as i32 + 1);
            } else {
                // We encountered digits then an alpha => invalid
                return -1;
            }
        } else if is_digit(ch) {
            // Now we parse row digits
            cell_row = cell_row * 10 + (ch as i32 - '0' as i32);
            dfound = true;
        } else {
            return -1;
        }
    }

    // Convert from 1-based to 0-based indices
    cell_col -= 1;
    cell_row -= 1;

    if cell_col < 0 || cell_row < 0 || cell_col >= cols || cell_row >= rows {
        -1
    } else {
        cell_row * cols + cell_col
    }
}

/// Return an integer representing the operator type (+, -, *, /).
fn return_optype(op: Option<char>) -> i32 {
    match op {
        Some('+') => 1,
        Some('-') => 2,
        Some('*') => 3,
        Some('/') => 4,
        _ => i32::MIN,
    }
}

/// A transactional value function that processes formulas of the form <cell> = <value>
/// or <cell> = <someOtherCell>.
/// If recalc (which checks for cycles) fails, the update is completely rejected.
fn value_func(
    formula: &str,
    c: i32,
    r: i32,
    pos_equal: i32,
    pos_end: i32,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula]
) -> i32 {
    // Example: formula might be "A1 = 42" or "A1 = B2"
    let ref_sub = &formula[..pos_equal as usize];
    let first = cell_parser(ref_sub, c, r, 0, pos_equal - 1, graph);

    if first == -1 {
        // println!("Invalid destination cell");
        return 1;
    }

    // Save the current cell state.
    unsafe {
        OLD_VALUE = arr[first as usize];
        OLD_OP_TYPE = formula_array[first as usize].op_type;
        OLD_P1 = formula_array[first as usize].p1;
        OLD_P2 = formula_array[first as usize].p2;
    }

    // Remove any existing dependencies for this cell.
    if formula_array[first as usize].op_type > 0 {
        delete_edge(graph, first, c, formula_array);
    }

    let mut second;
    let mut is_cell = 0;
    let mut is_negative = 0;
    let mut pos_eq = pos_equal;
    let x = formula.chars().nth((pos_eq + 1) as usize).unwrap();

    // Check for a leading sign.
    if x == '-' || x == '+' {
        if x == '-' {
            is_negative = 1;
        }
        pos_eq += 1;
    }

    // Distinguish literal number vs. cell reference.
    let next_char = formula[(pos_eq+1) as usize..].chars().next().unwrap();
    if is_digit(next_char) {
        // Numeric literal.
        let tmp = formula[(pos_eq + 1) as usize..].trim();

        // Check for invalid extra content: pure integer only
        if let Ok(val) = tmp.parse::<i32>() {
            second = val;
        } else if tmp.chars().all(|ch| ch.is_ascii_alphanumeric()) {
            // fallback – it might be a malformed cell
            second = -1;
        } else {
            //println!("Invalid literal or malformed expression: {}", tmp);
            return 1;
        }
    } else {
        // Cell reference.
        let ref_part = formula[(pos_eq + 1) as usize..].trim();
        second = cell_parser(ref_part, c, r, pos_eq + 1, pos_end - 1, graph);
        is_cell = 1;
    }

    if second == -1 {
        //println!("Invalid cell reference");
        return 1;
    }

    if is_negative == 1 && is_cell == 0 {
        second = -second;
    }

    // Apply the formula.
    if is_cell == 0 {
        arr[first as usize] = second;
        add_formula(graph, first, second, 0, 0, formula_array);
    } else {
        if is_negative == 1 {
            let tmp = -1 * arr[second as usize];
            arr[first as usize] = tmp;
            graph.adj[second as usize] = Some(add_edge(first, graph.adj[second as usize].take()));
            add_formula(graph, first, second, -1, 3, formula_array);
        } else {
            let tmp = arr[second as usize];
            arr[first as usize] = tmp;
            graph.adj[second as usize] = Some(add_edge(first, graph.adj[second as usize].take()));
            add_formula(graph, first, second, 0, 1, formula_array);
        }
    }

    // Attempt to recalculate downstream formulas.
    let b = recalculate(graph, c, arr, first, formula_array);
    if !b {
        // If recalculation fails (circular dependency detected), revert.
        //println!("Formula rejected due to circular dependency. Reverting changes.");
        arr[first as usize] = unsafe { OLD_VALUE };
        delete_edge(graph, first, c, formula_array);
        unsafe {
            formula_array[first as usize].op_type = OLD_OP_TYPE;
            formula_array[first as usize].p1 = OLD_P1;
            formula_array[first as usize].p2 = OLD_P2;
        }
        add_edge_formula(graph, first, c, formula_array);
        return 1;
    }

    // If any external revert trigger (HAS flag) is set, revert too.
    unsafe {
        if HAS != 0 {
            arr[first as usize] = OLD_VALUE;
            delete_edge(graph, first, c, formula_array);
            formula_array[first as usize].op_type = OLD_OP_TYPE;
            formula_array[first as usize].p1 = OLD_P1;
            formula_array[first as usize].p2 = OLD_P2;
            add_edge_formula(graph, first, c, formula_array);
        }
    }
    return 0;
}

/// Handle arithmetic expressions like "A1 = 5+10" or "B2 = C1 + D3".
/// Similar transactional logic (check recalc before committing) should be applied.
fn arth_op(
    formula: &str,
    c: i32,
    r: i32,
    pos_equal: i32,
    _pos_end: i32,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula]
) -> i32{
    let res;
    let mut notvalid = 0;
    let mut op: Option<char> = None;
    let mut opind = -1;
    let mut pt = pos_equal + 1;

    // Find the operator (+, -, *, /)
    for ch in formula[(pos_equal + 1) as usize..].chars() {
        if ch == '+' || ch == '-' || ch == '*' || ch == '/' {
            op = Some(ch);
            opind = pt;
            break;
        }
        pt += 1;
    }

    if opind == -1 {
        //println!("Invalid arithmetic input");
        return 1;
    }

    // Parse the left operand.
    let mut is1cell = 0;
    let mut is1num = 0;
    let mut sign1 = 1;
    let mut cell1 = String::new();
    let mut num1 = String::new();
    let mut l1 = 0;

    let mut start_pos = pos_equal + 1;
    let first_ch = formula.chars().nth(start_pos as usize).unwrap();
    if first_ch == '-' {
        sign1 = -1;
        start_pos += 1;
    } else if first_ch == '+' {
        start_pos += 1;
    }

    for ch in formula[start_pos as usize..opind as usize].chars() {
        if !is_digit(ch) && !is_alpha(ch) {
            notvalid = 1;
            break;
        } else if is1num != 0 && is_alpha(ch) {
            notvalid = 1;
            break;
        } else if is1cell != 0 {
            cell1.push(ch);
            l1 += 1;
        } else if is1num != 0 {
            num1.push(ch);
            l1 += 1;
        } else if is_alpha(ch) {
            is1cell = 1;
            cell1.push(ch);
            l1 += 1;
        } else if is_digit(ch) {
            is1num = 1;
            num1.push(ch);
            l1 += 1;
        }
    }

    if is1cell != 0 {
        let mut dfound = 0;
        for cch in cell1.chars() {
            if is_digit(cch) && dfound == 0 {
                dfound = 1;
            } else if !is_digit(cch) && dfound == 1 {
                notvalid = 1;
            }
        }
    }

    // Parse the right operand.
    let mut is2cell = 0;
    let mut is2num = 0;
    let mut sign2 = 1;
    let mut cell2 = String::new();
    let mut num2 = String::new();
    let mut l2 = 0;

    let mut second_start = opind + 1;
    let second_ch = formula.chars().nth(second_start as usize).unwrap();
    if second_ch == '-' {
        sign2 = -1;
        second_start += 1;
    } else if second_ch == '+' {
        second_start += 1;
    }

    for ch in formula[second_start as usize..].chars() {
        if !is_digit(ch) && !is_alpha(ch) {
            notvalid = 1;
            break;
        } else if is2num != 0 && is_alpha(ch) {
            notvalid = 1;
            break;
        } else if is2cell != 0 {
            cell2.push(ch);
            l2 += 1;
        } else if is2num != 0 {
            num2.push(ch);
            l2 += 1;
        } else if is_alpha(ch) {
            is2cell = 1;
            cell2.push(ch);
            l2 += 1;
        } else if is_digit(ch) {
            is2num = 1;
            num2.push(ch);
            l2 += 1;
        }
    }

    if is2cell != 0 {
        let mut ddfound = 0;
        for cc in cell2.chars() {
            if is_digit(cc) && ddfound == 0 {
                ddfound = 1;
            } else if !is_digit(cc) && ddfound == 1 {
                notvalid = 1;
            }
        }
    }

    if notvalid == 1 || (is1cell == 0 && is1num == 0) || (is2cell == 0 && is2num == 0) {
        //println!("Invalid arithmetic command\n");
        return 1;
    }

    // Evaluate left operand (call it second_cell)
    let mut second_cell;
    if is1cell != 0 {
        second_cell = cell_parser(&cell1, c, r, 0, l1 - 2, graph);
        if second_cell == -1 {
            //println!("Invalid cell reference in arithmetic (left operand)\n");
            return 1;
        }
        second_cell = sign1 * second_cell;
    } else {
        second_cell = num1.parse::<i32>().unwrap() * sign1;
    }

    // Evaluate right operand (call it third_cell)
    let mut third_cell;
    if is2cell != 0 {
        third_cell = cell_parser(&cell2, c, r, 0, (l2 - 2) as i32, graph);
        if third_cell == -1 {
            //println!("Invalid cell reference in arithmetic (right operand)\n");
            return 1;
        }
        third_cell = sign2 * third_cell;
    } else {
        third_cell = num2.parse::<i32>().unwrap() * sign2;
    }

    // Evaluate the destination cell (left of '=')
    let ref_sub = &formula[..pos_equal as usize];
    let first_cell = cell_parser(ref_sub, c, r, 0, pos_equal - 1, graph);
    if first_cell == -1 {
        //println!("Invalid destination cell in arithmetic\n");
        return 1;
    }

    unsafe {
        OLD_VALUE = arr[first_cell as usize];
        OLD_OP_TYPE = formula_array[first_cell as usize].op_type;
        OLD_P1 = formula_array[first_cell as usize].p1;
        OLD_P2 = formula_array[first_cell as usize].p2;
    }

    // Remove existing formula, if any
    if formula_array[first_cell as usize].op_type > 0 {
        delete_edge(graph, first_cell, c, formula_array);
    }

    // Now handle combinations of cell vs number operands.
    if is1cell == 0 && is2cell == 0 {
        // Both sides are numeric.
        res = arith(second_cell, third_cell, op);
        arr[first_cell as usize] = res;
        add_formula(graph, first_cell, res, 0, 0, formula_array);
    } else if is1cell == 1 && is2cell == 0 {
        // Left operand is a cell, right is numeric.
        let val_left = arr[second_cell as usize];
        res = arith(val_left, third_cell, op);
        arr[first_cell as usize] = res;
        graph.adj[second_cell as usize] = Some(add_edge(first_cell, graph.adj[second_cell as usize].take()));
        add_formula(graph, first_cell, second_cell, third_cell, return_optype(op), formula_array);
    } else if is1cell == 0 && is2cell == 1 {
        // Left operand is numeric, right is a cell.
        let val_right = arr[third_cell as usize];
        res = arith(second_cell, val_right, op);
        arr[first_cell as usize] = res;
        graph.adj[third_cell as usize] = Some(add_edge(first_cell, graph.adj[third_cell as usize].take()));
        add_formula(graph, first_cell, third_cell, second_cell, return_optype(op), formula_array);
    } else {
        // Both operands are cell references.
        let val_left = arr[second_cell as usize];
        let val_right = arr[third_cell as usize];
        res = arith(val_left, val_right, op);
        arr[first_cell as usize] = res;
        graph.adj[second_cell as usize] = Some(add_edge(first_cell, graph.adj[second_cell as usize].take()));
        graph.adj[third_cell as usize] = Some(add_edge(first_cell, graph.adj[third_cell as usize].take()));
        add_formula(graph, first_cell, second_cell, third_cell, return_optype(op), formula_array);
        // Additional formula edge (e.g., for specific arithmetic operations)
        add_formula(graph, first_cell, second_cell, third_cell, return_optype(op) + 4, formula_array);
    }

    // Attempt recalculation.
    let b = recalculate(graph, c, arr, first_cell, formula_array);
    if !b {
        // If recalculation (and thus dependency validation) fails, revert changes.
        //println!("Formula rejected due to circular dependency. Reverting changes.");
        arr[first_cell as usize] = unsafe { OLD_VALUE };
        delete_edge(graph, first_cell, c, formula_array);
        unsafe {
            formula_array[first_cell as usize].op_type = OLD_OP_TYPE;
            formula_array[first_cell as usize].p1 = OLD_P1;
            formula_array[first_cell as usize].p2 = OLD_P2;
        }
        add_edge_formula(graph, first_cell, c, formula_array);
        return 1;
    }

    unsafe {
        if HAS != 0 {
            arr[first_cell as usize] = OLD_VALUE;
            delete_edge(graph, first_cell, c, formula_array);
            formula_array[first_cell as usize].op_type = OLD_OP_TYPE;
            formula_array[first_cell as usize].p1 = OLD_P1;
            formula_array[first_cell as usize].p2 = OLD_P2;
            add_edge_formula(graph, first_cell, c, formula_array);
        }
    }
    return 0;
}

/// Handle functions like SUM, AVG, MIN, MAX, STDEV, SLEEP.
fn funct(
    formula: &str,
    c: i32,
    r: i32,
    pos_equal: i32,
    pos_end: i32,
    arr: &mut [i32],
    graph: &mut Graph,
    formula_array: &mut [Formula]
) -> i32 {
    let ref_sub = &formula[..pos_equal as usize];
    let first = cell_parser(ref_sub, c, r, 0, pos_equal - 1, graph);

    if first == -1 {
        //println!("Invalid destination cell in function");
        return 1;
    }

    unsafe {
        OLD_VALUE = arr[first as usize];
        OLD_OP_TYPE = formula_array[first as usize].op_type;
        OLD_P1 = formula_array[first as usize].p1;
        OLD_P2 = formula_array[first as usize].p2;
    }

    if formula_array[first as usize].op_type > 0 {
        delete_edge(graph, first, c, formula_array);
    }

    let open_paren1 = formula[pos_equal as usize..].find('(');
    let close_paren1 = formula[pos_equal as usize..].find(')');
    

    if let (Some(open), Some(close)) = (open_paren1, close_paren1) {
        if close <= open + 1 {
            //println!("Invalid range: Missing or misplaced parentheses\n");
            return 1;
        }
        let _formula_part = &formula[pos_equal as usize..];
        if (pos_equal as usize + close + 1) < formula.len() {
            // There's content after the closing parenthesis
            //println!("Invalid formula: Unexpected content after function\n");
            return 1;
        }
        // We extract the inside of the parentheses if needed.
        let _inside: &str = &formula[(pos_equal + open as i32 + 1) as usize..(pos_equal + close as i32) as usize];
    } else {
        //println!("Invalid range: Missing or misplaced parentheses\n");
        return 1;
    }
    let mut flag=true;
    let idx_open = pos_equal + open_paren1.unwrap() as i32;
    if idx_open - pos_equal >= 3 {
        // For functions with names of fixed length.
        if idx_open - pos_equal - 1 == 5 {
            if formula[(pos_equal + 1) as usize..].starts_with("STDEV") {
                flag=standard_dev_func(formula, c, r, pos_equal as usize, pos_end as usize, arr, graph, formula_array);
            } else if formula[(pos_equal + 1) as usize..].starts_with("SLEEP") {
                flag=sleep_func(formula, c, r, pos_equal as usize, pos_end as usize, arr, graph, formula_array);
            }
        } else if idx_open - pos_equal - 1 == 3 {
            if formula[(pos_equal + 1) as usize..].starts_with("MIN") {
                flag=min_func(formula, c, r, pos_equal as usize, pos_end as usize, arr, graph, formula_array);
            } else if formula[(pos_equal + 1) as usize..].starts_with("MAX") {
                //println!("hey");
                flag=max_func(formula, c, r, pos_equal as usize, pos_end as usize, arr, graph, formula_array);
            } else if formula[(pos_equal + 1) as usize..].starts_with("AVG") {
                flag=avg_func(formula, c, r, pos_equal as usize, pos_end as usize, arr, graph, formula_array);
            } else if formula[(pos_equal + 1) as usize..].starts_with("SUM") {
                flag=sum_func(formula, c, r, pos_equal as usize, pos_end as usize, arr, graph, formula_array);
            } else {
                //println!("Invalid function\n");
                return 1;
            }
        } else {
            //println!("Invalid function\n");
            return 1;
        }
    } else {
        //println!("Invalid function\n");
        return 1;
    }
    if !flag {
        return 1;
    }
    let b = recalculate(graph, c, arr, first, formula_array);
    if !b {
        //println!("Formula rejected due to circular dependency. Reverting changes.");
        arr[first as usize] = unsafe { OLD_VALUE };
        delete_edge(graph, first, c, formula_array);
        unsafe {
            formula_array[first as usize].op_type = OLD_OP_TYPE;
            formula_array[first as usize].p1 = OLD_P1;
            formula_array[first as usize].p2 = OLD_P2;
        }
        add_edge_formula(graph, first, c, formula_array);
        return 1;
    }

    unsafe {
        if HAS != 0 {
            arr[first as usize] = OLD_VALUE;
            delete_edge(graph, first, c, formula_array);
            formula_array[first as usize].op_type = OLD_OP_TYPE;
            formula_array[first as usize].p1 = OLD_P1;
            formula_array[first as usize].p2 = OLD_P2;
            add_edge_formula(graph, first, c, formula_array);
        }
    }
    return 0;
}

/// Main parser — called from your WASM code with parser(&mut s, formula).
/// Determines the formula type (simple value, arithmetic, or function) then dispatches.
pub fn parser(spreadsheet: &mut Spreadsheet, formula: &str) -> i32 {
    // Check if the formula is for movement keys.
    let first_char = formula.chars().next().unwrap();
    if ['w', 'a', 's', 'd'].contains(&first_char) {
        // Handle movement if necessary...
        // For now, we do nothing.
    }

    let c = spreadsheet.cols as i32;
    let r = spreadsheet.rows as i32;
    let arr = &mut spreadsheet.arr;
    let graph = &mut spreadsheet.graph;
    let formula_array = &mut spreadsheet.formula_array;

    // Find the position of '='.
    let mut pos_equal = 1000;
    let mut pos_end = 1000;
    let mut pt = 0;
    for ch in formula.chars() {
        if ch == '=' && pos_equal == 1000 {
            pos_equal = pt;
        }
        pos_end += 1;
        pt += 1;
    }

    if pos_equal == 1000 {
        // No '=' character means not a formula.
        return -1;
    }

    // Determine the formula type using basic heuristics.
    let mut value = 0;
    let mut arth_exp = 0;
    let mut func = 0;
    let mut found_digit = 0;
    for ch in formula[(pos_equal + 1) as usize..].chars() {
        if ch == '(' {
            func = 1;
            break;
        }
        if is_digit(ch) {
            found_digit = 1;
        }
        if (ch == '+' || ch == '-' || ch == '*' || ch == '/') && found_digit == 1 {
            arth_exp = 1;
            break;
        }
    }
    if func == 1 && arth_exp == 1 {
        //println!("Invalid input: can't be both function and arithmetic\n");
        return 1;
    }
    if func == 0 && arth_exp == 0 {
        value = 1;
    }
    let mut status1 : i32 = 1;
    // Dispatch based on the formula type.
    if value == 1 {
        status1 = value_func(formula, c, r, pos_equal, pos_end, arr, graph, formula_array);
    } else if arth_exp == 1 {
        status1 = arth_op(formula, c, r, pos_equal, pos_end, arr, graph, formula_array);
    } else if func == 1 {
        status1 = funct(formula, c, r, pos_equal, pos_end, arr, graph, formula_array);
    }

    if status1 == 0 {
        0
    } else {
        1
    }
}
