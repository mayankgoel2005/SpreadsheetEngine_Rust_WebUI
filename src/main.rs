use std::env;
use std::io::{self, Write};
use std::time::Instant;
mod spreadsheet;
mod input_parser;

const MAX_INPUT_SIZE: usize = 100;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("[0.0] (Usage: {} <rows> <cols>)", args[0]);
        return;
    }
    
    let rows: usize = match args[1].parse() {
        Ok(num) if (1..=1000).contains(&num) => num,
        _ => {
            println!("[0.0] (Error: Rows should be in the range 1 to 1000 inclusive)");
            return;
        }
    };
    
    let cols: usize = match args[2].parse() {
        Ok(num) if (1..=18278).contains(&num) => num,
        _ => {
            println!("[0.0] (Error: Cols should be in the range 1 to 18278 inclusive)");
            return;
        }
    };
    
    let mut spreadsheet = spreadsheet::initialize_spreadsheet(rows, cols);
    spreadsheet::print_spreadsheet(&spreadsheet);
    println!("[0.0] (ok) ");
    
    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();
        
        if io::stdin().read_line(&mut input).is_err() {
            println!("[0.0] (unrecognized cmd) ");
            break;
        }
        
        let start = Instant::now();
        if !input_parser::parse_input(input.trim(), &mut spreadsheet, start) {
            break;
        }
    }
}
