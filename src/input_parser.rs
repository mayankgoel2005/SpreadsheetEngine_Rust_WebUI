use std::time::Instant;
use crate::spreadsheet::Spreadsheet;
use crate::graph::{Graph, Formula, arith, add_formula, add_edge, delete_edge, recalculate, add_edge_formula};
use crate::functions::{max_func, sum_func, standard_dev_func, avg_func, min_func, sleep_func};
static mut OLD_VALUE: i32 = 0;
static mut OLD_OP_TYPE: i32 = 0;
static mut OLD_P1: i32 = 0;
static mut OLD_P2: i32 = 0;
pub static mut HAS: i32 = 0;

fn is_alpha(c: char) -> bool {
    c >= 'A' && c <= 'Z'
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub fn cell_parser(a: &str, c: i32, r: i32, start: i32, end: i32, _graph: &mut Graph) -> i32 {
    let mut cell_col = 0;
    let mut cell_row = 0;
    let mut dfound = false;

    for i in a.chars(){
        println!("x={}",i);
        if is_alpha(i) {
            if !dfound {
                cell_col = cell_col * 26 + (i as i32 - 'A' as i32 + 1);
                println!("cell_col={}",cell_col);

            } else {
                return -1;
            }
        } else if is_digit(i) {
            cell_row = cell_row * 10 + (i as i32 - '0' as i32);
            dfound = true;
        } else {
            return -1;
        }
    }
    cell_col -= 1;
    cell_row -= 1;
    println!("{}  {}  {}  {}",c,r, cell_col, cell_row);
    if cell_col < 0 || cell_row < 0 || cell_col >= c || cell_row >= r {
        -1
    } else {
        cell_row * c + cell_col
    }
}

pub fn return_optype(op: Option<char>) -> i32 {
    match op {
        Some('+') => 1,
        Some('-') => 2,
        Some('*') => 3,
        Some('/') => 4,
        _ => i32::MIN,
    }
}

pub fn value_func(a: &str, c: i32, r: i32, pos_equal: i32, pos_end: i32, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let ref_sub=&a[..pos_equal as usize];
    let first=cell_parser(ref_sub,c,r,0,pos_equal-1,graph);
    println!("first={}",first);
    if first==-1 {
        println!("invalid cell");
        return;
    }
    unsafe {
        OLD_VALUE = arr[first as usize];
        OLD_OP_TYPE = formula_array[first as usize].op_type;
        OLD_P1 = formula_array[first as usize].p1;
        OLD_P2 = formula_array[first as usize].p2;
    }
    if formula_array[first as usize].op_type>0 {
        delete_edge(graph,first,c,formula_array);
    }
    let mut second=-1;
    let mut is_cell=0;
    let mut is_negative=0;
    let mut pos_eq=pos_equal;
    let x=a.chars().nth(pos_eq as usize + 1).unwrap();
    if x=='-' || x=='+' {
        if x=='-' {
            is_negative=1;
        }
        pos_eq+=1;
    }
    if is_digit(x) {
        println!("{}",x);
        let tmp: &str = &a[(pos_eq + 1) as usize..];
        second = tmp.trim().parse().unwrap_or(0);
        println!("second={}",second);
    }else{
        second=cell_parser(a,c,r,pos_eq+1,pos_end-1,graph);
        is_cell=1;
    }
    if second==-1 {
        println!("invalid cell");
        return;
    }
    if is_negative==1 && is_cell==0 {
        second=-second;
    }
    if is_cell == 0 {
        arr[first as usize] = second;
        add_formula(graph, first, second, 0, 0,formula_array);
        recalculate(graph, c, arr, first,formula_array);
    }
    else {
        if is_negative==1 {
            let tmp = -1*arr[second as usize];
            arr[first as usize] = tmp;
            graph.adj[second as usize] = Some(add_edge(first, graph.adj[second as usize].take()));
            add_formula(graph, first, second, -1, 3,formula_array);
            recalculate(graph,c,arr,first,formula_array);
        }
        else{
            println!("heyu");
            let tmp = arr[second as usize];
            arr[first as usize] = tmp;
            graph.adj[second as usize] = Some(add_edge(first, graph.adj[second as usize].take()));
            add_formula(graph, first, second, 0, 1, formula_array);
            recalculate(graph, c, arr, first, formula_array);
        }
    }
    unsafe {
        if unsafe{HAS!=0} {
            arr[first as usize] = OLD_VALUE;
            delete_edge(graph, first, c, formula_array);

            formula_array[first as usize].op_type = OLD_OP_TYPE;
            formula_array[first as usize].p1 = OLD_P1;
            formula_array[first as usize].p2 = OLD_P2;

            add_edge_formula(graph, first, c, formula_array);
        }
    }
}

fn arth_op(a: &str, c: i32, r: i32, pos_equal: i32, pos_end: i32, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let mut res=0;
    let mut notvalid=0;
    let mut op: Option<char> = None;
    let mut opind=-1;
    let mut pt=pos_equal+1;
    for ch in a[(pos_equal+1) as usize..].chars() {
        println!("ch={} {}",ch,pt);
        if ch=='+'|| ch=='-' || ch=='*' || ch=='/' {
            op = Some(ch);
            opind = pt;
            break;
        }
        pt+=1;
        println!("{}",opind);
    }
    if opind==-1 {
        println!("Invalid Sinput");
        return;
    }
    let mut is1cell = 0;
    let mut is1num = 0;
    let mut sign1 = 1;
    let mut cell1=String::new();
    let mut num1=String::new();
    let mut l1 = 0;

    let mut start_pos = pos_equal + 1;
    let ch=a.chars().nth(start_pos as usize).unwrap();
    if ch == '-' {
        sign1 = -1;
        start_pos+=1;
    }
    else if ch == '+' {
        start_pos+=1;
    }
    for ch in a[start_pos as usize..opind as usize].chars() {
        if !is_digit(ch) && !is_alpha(ch) {
            notvalid = 1;
            break;
        }
        else if is1num!=0 && is_alpha(ch) {
            notvalid = 1;
            break;
        }
        else if is1cell!=0 {
            cell1.push(ch);
            l1+=1;
        }
        else if is1num!=0 {
            num1.push(ch);
            l1+=1;
        }
        else if is_alpha(ch) {
            is1cell = 1;
            cell1.push(ch);
            l1+=1;
        }
        else if is_digit(ch) {
            is1num = 1;
            num1.push(ch);
            l1+=1;
        }
    }
    println!("{}dd",cell1);
    if is1cell!=0 {
        let mut ii=0;
        let mut dfound = 0;
        while cell1.chars().nth(ii).is_some() {
            if is_digit(cell1.chars().nth(ii).unwrap()) && dfound==0 {
                dfound = 1;
            }
            else if !is_digit(cell1.chars().nth(ii).unwrap()) && dfound==1 {
                notvalid = 1;
            }
            ii+=1;
        }
    }
    let mut is2cell = 0;
    let mut is2num = 0;
    let mut sign2 = 1;
    let mut cell2=String::new();
    let mut num2=String::new();
    let mut l2 = 0;

    let mut second_start = opind + 1;
    let ch=a.chars().nth(second_start as usize).unwrap();
    if ch == '-' {
        sign2 = -1;
        second_start+=1;
    }
    else if ch == '+' {
        second_start+=1;
    }
    for ch in a[second_start as usize..].chars() {
        if !is_digit(ch) && !is_alpha(ch) {
            notvalid = 1;
            break;
        }
        else if is2num!=0 && is_alpha(ch) {
            notvalid = 1;
            break;
        }
        else if is2cell!=0 {
            cell2.push(ch);
            l2+=1;
        }
        else if is2num!=0 {
            num2.push(ch);
            l2+=1;
        }
        else if is_alpha(ch) {
            is2cell = 1;
            cell2.push(ch);
            l2+=1;
        }
        else if is_digit(ch) {
            is2num = 1;
            num2.push(ch);
            l2+=1;
        }
    }

    if is2cell!=0 {
        let mut ddfound = 0;
        for ch in cell2.chars() {
            if is_digit(ch) && ddfound==0 {
                ddfound = 1;
            }
            else if !is_digit(ch) && ddfound==1 {
                notvalid = 1;
            }
        }
    }

    if notvalid==1 || (is1cell==0 && is1num==0) || (is2cell==0 && is2num==0) {
        println!("Invalid command\n");
        return;
    }
    let mut second_cell=-1;
    if is1cell!=0 {
        println!("{} g",l1);
        second_cell = cell_parser(&cell1, c, r, 0, l1 - 2, graph);
        if second_cell == -1 {
            println!("Invalid cell Areference\n");
            return;
        }
    }
    else {
        println!("second_cell={}",num1);
        second_cell = num1.parse::<i32>().unwrap() * sign1;
    }
    let mut third_cell=-1;
    if is2cell!=0 {
        third_cell = cell_parser(&cell2, c, r, 0, (l2 - 2) as i32, graph);
        if third_cell == -1 {
            println!("Invalid cell Breference\n");
            return;
        }
    }
    else {
        third_cell = num2.parse::<i32>().unwrap() * sign2;
    }
    let ref_sub=&a[..pos_equal as usize];
    println!("ref_sub={}",ref_sub);
    let first_cell = cell_parser(ref_sub, c, r, 0, pos_equal - 1, graph);
    if first_cell == -1 {
        println!("Invalid cell Creference\n");
        return;
    }
    println!("{} {} {}",first_cell,second_cell,third_cell);
    unsafe {
        OLD_VALUE = arr[first_cell as usize];
        OLD_OP_TYPE = formula_array[first_cell as usize].op_type;
        OLD_P1 = formula_array[first_cell as usize].p1;
        OLD_P2 = formula_array[first_cell as usize].p2;
    }

    if formula_array[first_cell as usize].op_type > 0 {
        delete_edge(graph, first_cell, c, formula_array);
    }

    if is1cell==0 && is2cell==0 {
        res = arith(second_cell, third_cell, Some(op).expect("Invalid Ainput"));
        arr[first_cell as usize] = res;
        add_formula(graph, first_cell, res, 0, 0,formula_array);
    }
    else if is1cell==1 && is2cell==0 {
        res = arith(arr[second_cell as usize], third_cell, Some(op).expect("Invalid Binput"));
        arr[first_cell as usize] = res;
        graph.adj[second_cell as usize] = Some(add_edge(first_cell, graph.adj[second_cell as usize].take()));
        add_formula(graph, first_cell, second_cell, third_cell, return_optype(op),formula_array);
    }
    else if is1cell==0 && is2cell==1 {
        res = arith(second_cell, arr[third_cell as usize], Some(op).expect("Invalid Dinput"));
        arr[first_cell as usize] = res;
        graph.adj[third_cell as usize] = Some(add_edge(first_cell, graph.adj[third_cell as usize].take()));
        add_formula(graph, first_cell, third_cell, second_cell, return_optype(op),formula_array);
    }
    else {
        println!("{} UH",second_cell);
        res = arith(arr[second_cell as usize], arr[third_cell as usize], Some(op).expect("Invalid TRWERWEinput"));
        arr[first_cell as usize] = res;
        graph.adj[second_cell as usize] = Some(add_edge(first_cell, graph.adj[second_cell as usize].take()));
        graph.adj[third_cell as usize] = Some(add_edge(first_cell, graph.adj[third_cell as usize].take()));
        add_formula(graph, first_cell, second_cell, third_cell, return_optype(op),formula_array);
        add_formula(graph, first_cell, second_cell, third_cell, return_optype(op) + 4,formula_array);
    }
    recalculate(graph, c, arr, first_cell,formula_array);
    unsafe {
        if unsafe {HAS==1} {
            arr[first_cell as usize] = OLD_VALUE;
            delete_edge(graph, first_cell, c, formula_array);

            formula_array[first_cell as usize].op_type = OLD_OP_TYPE;
            formula_array[first_cell as usize].p1 = OLD_P1;
            formula_array[first_cell as usize].p2 = OLD_P2;

            add_edge_formula(graph, first_cell, c, formula_array);
        }
    }
}

fn funct(a: &str, c: i32, r: i32, pos_equal: i32, pos_end: i32, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) {
    let ref_sub=&a[..pos_equal as usize];
    let first=cell_parser(ref_sub,c,r,0,pos_equal-1,graph);
    if first == -1 {
        println!("Invalid cellM");
        return;
    }
    unsafe {
        OLD_VALUE = arr[first as usize];
        OLD_OP_TYPE = formula_array[first as usize].op_type;
        OLD_P1 = formula_array[first as usize].p1;
        OLD_P2 = formula_array[first as usize].p2;
    }

    if formula_array[first as usize].op_type > 0 {
        delete_edge(graph, first, c,formula_array);
    }
    let open_paren1 = a[pos_equal as usize..].find('(');
    let close_paren1 = a[pos_equal as usize..].find(')');

    if let (Some(open), Some(close)) = (open_paren1, close_paren1) {
        if close <= open + 1 {
            println!("Invalid range: Missing or misplaced parentheses\n");
            return;
        }

        let open_idx = pos_equal + <usize as TryInto<i32>>::try_into(open).unwrap();
        let close_idx: i32 = pos_equal + <usize as TryInto<i32>>::try_into(close).unwrap();
        let inside: &str = &a[(open_idx + 1) as usize..close_idx as usize];
        println!("Inside parentheses: {}", inside);
    } else {
        println!("Invalid range: Missing or misplaced parentheses\n");
        return;
    }
    let idx_open = pos_equal + <usize as TryInto<i32>>::try_into(open_paren1.unwrap()).unwrap();
    if idx_open - pos_equal >= 3 {
        if idx_open - pos_equal - 1 == 5 {
            if a[(pos_equal + 1) as usize..].starts_with("STDEV") {
                standard_dev_func(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph,formula_array);
                recalculate(graph, c, arr, first,formula_array);
            }
            else if a[(pos_equal + 1) as usize..].starts_with("SLEEP") {
                sleep_func(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph,formula_array);
                recalculate(graph, c, arr, first,formula_array);
            }
        }
        else if idx_open - pos_equal - 1 == 3 {
            if a[(pos_equal + 1) as usize..].starts_with("MIN") {
                min_func(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph, formula_array);
                recalculate(graph, c, arr, first, formula_array);
            }
            else if a[(pos_equal + 1) as usize..].starts_with("MAX") {
                max_func(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph,formula_array);
                recalculate(graph, c, arr, first,formula_array);
            }
            else if a[(pos_equal + 1) as usize..].starts_with("AVG") {
                avg_func(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph,formula_array);
                recalculate(graph, c, arr, first, formula_array);
            }
            else if a[(pos_equal + 1) as usize..].starts_with("SUM") {
                sum_func(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph,formula_array);
                recalculate(graph, c, arr, first,formula_array);
            }
        }
        else {
            println!("Invalid function\n");
        }
    }
    else {
        println!("Invalid function\n");
    }
    unsafe {
        if unsafe{HAS!=0} {
            arr[first as usize] = OLD_VALUE;
            delete_edge(graph, first, c, formula_array);
            formula_array[first as usize].op_type = OLD_OP_TYPE;
            formula_array[first as usize].p1 = OLD_P1;
            formula_array[first as usize].p2 = OLD_P2;
            add_edge_formula(graph, first, c, formula_array);
        }
    }
}

pub fn parser(a: &str, c: i32, r: i32, arr: &mut [i32], graph: &mut Graph, formula_array: &mut [Formula]) -> i32 {
    let x=a.chars().nth(0).unwrap();
    println!("{}",a);
    if x == 'w' || x == 'd' || x == 'a' || x == 's' {

    }
    let mut pos_equal = 1000;
    let mut pos_end = 1000;
    let mut pt=0;
    for i in a.chars() {
        if i=='=' && pos_equal==1000 {
            pos_equal=pt;
        }
        pos_end+=1;
        pt+=1;
    }
    if pos_equal == 1000 {
        return -1;
    }
    let mut value = 0;
    let mut arth_exp = 0;
    let mut func = 0;
    let mut found_digit = 0;
    for ch in a[(pos_equal+1) as usize..].chars() {
        if ch=='(' {
            func=1;
            break;
        }
        if is_digit(ch) {
            println!("hi");
            found_digit=1;
        }
        if (ch == '+' || ch == '-' || ch == '*' || ch == '/') && (found_digit == 1) {
            arth_exp = 1;
            break;
        }
    }
    if func == 1 && arth_exp == 1 {
        println!("Invalid Cinput");
        return -1;
    }
    if func == 0 && arth_exp == 0 {
        value = 1;
    }
    if value == 1 {
        value_func(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph, formula_array);
    }
    else if arth_exp == 1 {
        arth_op(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph,formula_array);
    }
    else if func == 1 {
        funct(a, c, r, pos_equal.try_into().unwrap(), pos_end.try_into().unwrap(), arr, graph,formula_array);
    }
    if value == 1 || func == 1 || arth_exp == 1 {
        1
    }
    else {
        0
    }
}