use std::rc::Rc;
use std::cell::RefCell;
const INT_MIN: i32 = i32::MIN;
const INT_MAX: i32 = i32::MAX;
#[derive(Debug)]
struct QueueNode {
    cell: i32,
    next: Option<Rc<RefCell<QueueNode>>>,
}

#[derive(Debug)]
struct Queue {
    front: Option<Rc<RefCell<QueueNode>>>,
    rear: Option<Rc<RefCell<QueueNode>>>,
}

impl Queue {
    fn new() -> Self {
        Queue {
            front: None,
            rear: None,
        }
    }

    fn enqueue(&mut self, cell: i32) {
        let new_node = Rc::new(RefCell::new(QueueNode { cell, next: None }));
        if self.rear.is_none() {
            self.front = Some(Rc::clone(&new_node));
            self.rear = Some(new_node);
        } else {
            if let Some(rear) = self.rear.take() {
                rear.borrow_mut().next = Some(Rc::clone(&new_node));
                self.rear = Some(new_node);
            }
        }
    }

    fn dequeue(&mut self) -> Option<i32> {
        if let Some(front) = self.front.take() {
            let result = front.borrow().cell;
            self.front = front.borrow_mut().next.take();
            if self.front.is_none() {
                self.rear = None;
            }
            Some(result)
        } else {
            None
        }
    }
}

fn get_nodes_from_avl(root: Option<&Box<Cell>>, nodes: &mut Vec<i32>) {
    if let Some(node) = root {
        get_nodes_from_avl(node.left.as_ref(), nodes);
        nodes.push(node.cell);
        get_nodes_from_avl(node.right.as_ref(), nodes);
    }
}
const CELLS: usize = 1000;

#[derive(Copy, Clone, Debug)]
struct Formula {
    op_type: i32,
    op_info1: usize,
    op_info2: usize,
}

#[derive(Debug)]
struct Cell {
    cell: i32,
    height: i32,
    left: Option<Box<Cell>>,
    right: Option<Box<Cell>>,
}

#[derive(Debug)]
struct Graph {
    adj: Vec<Option<Box<Cell>>>,
}

fn addformula(cell: i32, op_type: i32, p1: i32, p2: i32, formulalist: &mut [Formula]) {
    let mut f = Formula {
        op_type,
        op_info1: 0,
        op_info2: 0,
    };
    if op_type == 0 {
        f.op_info1 = p1 as usize;
    } else {
        f.op_info1 = p1 as usize;
        f.op_info2 = p2 as usize;
    }
    formulalist[cell as usize] = f;
}

fn height(cell: Option<&Box<Cell>>) -> i32 {
    cell.map_or(0, |c| c.height)
}

fn balance(cell: Option<&Box<Cell>>) -> i32 {
    cell.map_or(0, |c| height(c.left.as_ref()) - height(c.right.as_ref()))
}

fn right_rotate(mut cell: Box<Cell>) -> Box<Cell> {
    let mut x = cell.left.take().expect("left child must exist for rotation");
    let t2 = x.right.take();
    x.right = Some(cell);
    x.right.as_mut().unwrap().left = t2;
    let right = x.right.as_mut().unwrap();
    right.height = 1 + std::cmp::max(height(right.left.as_ref()), height(right.right.as_ref()));
    x.height = 1 + std::cmp::max(height(x.left.as_ref()), height(x.right.as_ref()));
    x
}

fn left_rotate(mut cell: Box<Cell>) -> Box<Cell> {
    let mut x = cell.right.take().expect("right child must exist for rotation");
    let t2 = x.left.take();
    x.left = Some(cell);
    x.left.as_mut().unwrap().right = t2;
    let left = x.left.as_mut().unwrap();
    left.height = 1 + std::cmp::max(height(left.left.as_ref()), height(left.right.as_ref()));
    x.height = 1 + std::cmp::max(height(x.left.as_ref()), height(x.right.as_ref()));
    x
}

fn addcell(cell: i32) -> Box<Cell> {
    Box::new(Cell {
        cell,
        left: None,
        right: None,
        height: 1,
    })
}

fn create_graph() -> Box<Graph> {
    Box::new(Graph {
        adj: vec![None; CELLS],
    })
}

fn addedge(cell1: i32, x: Option<Box<Cell>>) -> Box<Cell> {
    match x {
        None => addcell(cell1),
        Some(mut node) => {
            if cell1 < node.cell {
                node.left = Some(addedge(cell1, node.left));
            } else if cell1 > node.cell {
                node.right = Some(addedge(cell1, node.right));
            } else {
                return node;
            }
            let left_height = height(node.left.as_ref());
            let right_height = height(node.right.as_ref());
            node.height = 1 + left_height.max(right_height);

            let bal = balance(Some(&node));
            if bal > 1 && cell1 < node.left.as_ref().unwrap().cell {
                return right_rotate(node);
            }
            if bal < -1 && cell1 > node.right.as_ref().unwrap().cell {
                return left_rotate(node);
            }
            if bal > 1 && cell1 > node.left.as_ref().unwrap().cell {
                node.left = Some(left_rotate(node.left.take().unwrap()));
                return right_rotate(node);
            }
            if bal < -1 && cell1 < node.right.as_ref().unwrap().cell {
                node.right = Some(right_rotate(node.right.take().unwrap()));
                return left_rotate(node);
            }

            node
        }
    }
}

fn deletecell(cell1: i32, x: Option<Box<Cell>>) -> Option<Box<Cell>> {
    match x {
        None => None,
        Some(mut node) => {
            if cell1 < node.cell {
                node.left = deletecell(cell1, node.left);
            } else if cell1 > node.cell {
                node.right = deletecell(cell1, node.right);
            } else {
                if node.left.is_none() || node.right.is_none() {
                    return if node.left.is_some() {
                        node.left.take()
                    } else {
                        node.right.take()
                    };
                } else {
                    let min_val = {
                        let mut current = node.right.as_ref().unwrap();
                        while let Some(ref left) = current.left {
                            current = left;
                        }
                        current.cell
                    };
                    node.cell = min_val;
                    node.right = deletecell(min_val, node.right);
                }
            }

            let left_height = height(node.left.as_ref());
            let right_height = height(node.right.as_ref());
            node.height = 1 + left_height.max(right_height);

            let bal = balance(Some(&node));

            if bal > 1 && balance(node.left.as_ref()) >= 0 {
                return Some(right_rotate(node));
            }
            if bal > 1 && balance(node.left.as_ref()) < 0 {
                node.left = Some(left_rotate(node.left.take().unwrap()));
                return Some(right_rotate(node));
            }
            if bal < -1 && balance(node.right.as_ref()) <= 0 {
                return Some(left_rotate(node));
            }
            if bal < -1 && balance(node.right.as_ref()) > 0 {
                node.right = Some(right_rotate(node.right.take().unwrap()));
                return Some(left_rotate(node));
            }

            Some(node)
        }
    }
}

fn deleteedge(mut graph: Box<Graph>, cell: i32, cols: i32, formulalist: &[Formula]) {
    let f: Formula = formulalist[cell as usize];

    if f.op_type >= 1 && f.op_type <= 4 {
        graph.adj[f.op_info1] = deletecell(cell, graph.adj[f.op_info1].take());
    } else if f.op_type >= 5 && f.op_type <= 8 {
        graph.adj[f.op_info1] = deletecell(cell, graph.adj[f.op_info1].take());
        graph.adj[f.op_info2] = deletecell(cell, graph.adj[f.op_info2].take());
    } else if f.op_type >= 9 && f.op_type <= 13 {
        let start = f.op_info1;
        let end = f.op_info2;
        let strow = start / cols;
        let endrow = end / cols;
        let stcol = start % cols;
        let endcol = end % cols;
        for i in strow..=endrow {
            for j in stcol..=endcol {
                let index = (i * cols + j) as usize;
                graph.adj[index] = deletecell(cell, graph.adj[index].take());
            }
        }
    }
}

fn addedge_formula(graph: &mut Graph, cell: i32, cols: i32, formula_array: &[Formula]) {
    let x = formula_array[cell as usize];
    if x.op_type >= 1 && x.op_type <= 4 {
        // Note: Adjust field names according to your definition.
        graph.adj[x.op_info1] = Some(addedge(cell, graph.adj[x.op_info1].take()));
    } else if x.op_type >= 5 && x.op_type <= 8 {
        graph.adj[x.op_info1] = Some(addedge(cell, graph.adj[x.op_info1].take()));
        graph.adj[x.op_info2] = Some(addedge(cell, graph.adj[x.op_info2].take()));
    } else if x.op_type >= 9 && x.op_type <= 13 {
        let start_cell = x.op_info1 as i32;
        let end_cell = x.op_info2 as i32;
        let start_row = start_cell / cols;
        let start_col = start_cell % cols;
        let end_row = end_cell / cols;
        let end_col = end_cell % cols;
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let target_cell = row * cols + col;
                graph.adj[target_cell as usize] = Some(addedge(cell, graph.adj[target_cell as usize].take()));
            }
        }
    }
}

fn topo_sort(graph: &Graph, start: i32, size: &mut i32, hascycle: &mut i32) -> Option<i32> {
    *size = 0;
    *hascycle = 0;
    let num=1;
    let mut result = vec![0; CELLS];
    let mut indeg = vec![0; CELLS];
    let mut reachable = vec![0; CELLS];
    let mut q = Queue::new();
    let mut vis = Queue::new();
    vis.enqueue(start);
    reachable[start as usize] = 1;
    while(vis.front.is_some()){
        let cur = vis.dequeue().unwrap();
        if(graph.adj[cur as usize].is_some()) {
            let mut nodes = vec![0; CELLS];
            let ct = 0;
            get_nodes_from_avl(graph.adj[cur as usize].as_ref(), &mut nodes);
            for i in 0..(ct - 1) {
                let dep = nodes[i];
                indeg[dep]+ +;
                if (reachable[dep as usize] == 0) {
                    reachable[dep as usize] = 1;
                    num+ +;
                    vis.enqueue(dep);
                }
            }
        }
    }
    if(indeg[start as usize] > 0){
        println!("Cycle detected");
        return None;
    }
    q.enqueue(start);
    while(q.front.is_some()){
        let cur = q.dequeue().unwrap();
        let k= (*size)+1;
        result[k]=cur;
        if(graph.adj[cur].is_some()){
            let mut nodes = vec![0; CELLS];
            let ct=0;
            get_nodes_from_avl(graph.adj[cur as usize].as_ref(), &mut nodes);
            for i in 0..(ct - 1) {
                let dep = nodes[i];
                indeg[dep] -= 1;
                if (indeg[dep as usize] == 0) {
                    q.enqueue(dep);
                }
            }
        }
    }
    if(*size<num){
        *hascycle = 1;
        return None;
    }
    return result;
}

fn arith(v1: i32, v2: i32, op: char) -> i32 {
    match op {
        '+' => v1 + v2,
        '-' => v1 - v2,
        '*' => v1 * v2,
        '/' => if v2 != 0 { v1 / v2 } else { INT_MIN },
        _   => INT_MIN,
    }
}

fn recalc(graph: &Graph, c: i32, arr: &mut [i32], start_cell: i32, formula_array: &[Formula]) {
    let mut size=0;
    let mut hascycle;
    let sorted_cells = topo_sort(graph, start_cell, &mut size, &mut hascycle);
    if hascycle {
        println!("Error: Circular dependency detected. Command rejected.");
        return;
    }
    for i in 0..size {
        arr[sorted_cells[i]] = 0;
    }
    for &cell in &sorted_cells {
        let f = formula_array[cell];
        if f.op_type == 0 {
            if f.op_info1 == INT_MIN {
                println!(
                    "Error: Cell {} has an invalid constant value (INT_MIN)",
                    cell
                );
                arr[cell] = INT_MIN;
            } else {
                arr[cell] = f.op_info1;
            }
        } else if (1..=4).contains(&f.op_type) {
            let v1 = arr[f.op_info1 as usize];
            let v2 = f.op_info2;
            if v1 == INT_MIN {
                println!(
                    "Error: Cell {} has invalid operand (v1 is INT_MIN)",
                    f.op_info1
                );
                arr[cell] = INT_MIN;
                continue;
            }
            let op = match f.op_type {
                1 => '+',
                2 => '-',
                3 => '*',
                4 => '/',
                _ => unreachable!(),
            };
            if op == '/' && v2 == 0 {
                arr[cell] = INT_MIN;
                continue;
            }
            arr[cell] = arith(v1, v2, op);
        } else if (5..=8).contains(&f.op_type) {
            let v1 = arr[f.op_info1 as usize];
            let v2 = arr[f.op_info2 as usize];
            if f.op_type == 8 && v2 == 0 {
                arr[cell] = INT_MIN;
                continue;
            }
            if v1 == INT_MIN || v2 == INT_MIN {
                println!(
                    "Error: One of the operands for cell {} is INT_MIN",
                    cell
                );
                arr[cell] = INT_MIN;
                continue;
            }
            let op = match f.op_type {
                5 => '+',
                6 => '-',
                7 => '*',
                8 => '/',
                _ => unreachable!(),
            };
            arr[cell] = arith(v1, v2, op);
        } else if (9..=13).contains(&f.op_type) {
            let start_cell = f.op_info1;
            let end_cell = f.op_info2;
            let start_row = start_cell / C;
            let start_col = start_cell % C;
            let end_row = end_cell / C;
            let end_col = end_cell % C;

            let mut sum = 0;
            let mut count = 0;
            let mut stdev_squared = 0;
            let mut min_val = INT_MAX;
            let mut max_val = INT_MIN;
            let mut has_error = false;

            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    let idx = (row * C + col) as usize;
                    let val = arr[idx];
                    if val == INT_MIN {
                        has_error = true;
                        break;
                    }
                    sum += val;
                    count += 1;
                    if val < min_val {
                        min_val = val;
                    }
                    if val > max_val {
                        max_val = val;
                    }
                }
                if has_error {
                    break;
                }
            }

            if has_error {
                arr[cell] = INT_MIN;
                continue;
            }

            let mean = sum as f64 / count as f64;
            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    let idx = (row * C + col) as usize;
                    stdev_squared += ((arr[idx] as f64 - mean).powi(2)) as i32;
                }
            }

            arr[cell] = match f.op_type {
                9 => min_val,
                10 => max_val,
                11 => sum / count,
                12 => sum,
                13 => ( (stdev_squared as f64 / count as f64).sqrt() ) as i32,
                _ => unreachable!(),
            };
        } else if f.op_type == 14 {
            let sleep_value = if f.op_info1 == cell as i32 {
                f.op_info2
            } else {
                arr[f.op_info1 as usize]
            };

            if sleep_value == INT_MIN {
                println!("Error: Invalid sleep value in cell {}", cell);
                arr[cell] = INT_MIN;
                continue;
            } else if sleep_value <= 0 {
                arr[cell] = sleep_value;
                continue;
            }
            sleep(sleep_value); // Perform sleep.
            arr[cell] = sleep_value;
        }
    }
}