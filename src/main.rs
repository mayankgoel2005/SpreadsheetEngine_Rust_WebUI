use std::env;
use std::io::{self, Write};
use std::time::Instant;

mod spreadsheet;
mod input_parser;
mod test;
mod graph;
mod functions;
mod display;
mod scrolling;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <rows> <cols>", args[0]);
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
    spreadsheet.output_disabled = false;

    // Initial display if output is enabled
    if !spreadsheet.output_disabled {
        spreadsheet::print_spreadsheet(&spreadsheet);
    }
    let global_start = Instant::now();
    println!("[{:.6}] (ok)", global_start.elapsed().as_secs_f64());

    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();

        if io::stdin().read_line(&mut input).is_err() {
            println!("[{:.6}] (unrecognized cmd)", global_start.elapsed().as_secs_f64());
            break;
        }

        let trimmed = input.trim();
        if trimmed == "q" {
            break;
        }
        let cmd_start = Instant::now();
        let mut status = true;
        if trimmed == "disable_output" {
            spreadsheet.output_disabled = true;
            println!("[{:.6}] (ok)", cmd_start.elapsed().as_secs_f64());
            continue;
        } else if trimmed == "enable_output" {
            spreadsheet.output_disabled = false;
            spreadsheet::print_spreadsheet(&spreadsheet);
            println!("[{:.6}] (ok)", cmd_start.elapsed().as_secs_f64());
            continue;
        }

        if trimmed.starts_with("w")
            || trimmed.starts_with("a")
            || trimmed == "s"
            || trimmed == "d"
            || trimmed.starts_with("scroll_to ")
        {
            scrolling::scroller(trimmed, &mut spreadsheet);
        } else {
            status = input_parser::parse_input(trimmed, &mut spreadsheet, cmd_start);
        }

        let elapsed = cmd_start.elapsed().as_secs_f64();
        if !spreadsheet.output_disabled {
            spreadsheet::print_spreadsheet(&spreadsheet);
        }
        if status {
            println!("[{:.6}] (ok)", elapsed);
        } else {
            println!("[{:.6}] (unrecognized command)", elapsed);
        }
    }
}