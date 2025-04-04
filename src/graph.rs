use std::rc::Rc;
use std::cell::RefCell;

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