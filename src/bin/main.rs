// src/bin/main.rs
#[cfg(feature = "autograder")]

fn main() {
    use std::env;
    use std::io::{self, Write};
    use std::time::Instant;

    // Import modules from your library (the name here must match the package name in Cargo.toml)
    use lab1_2023CS10204_2023CS10076_2023CS10186::{
        spreadsheet, display, input_parser, graph, functions, scrolling,
    };

    // Original CLI code from your previous main.rs:
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <rows> <cols>", args[0]);
        return;
    }
    let rows: usize = match args[1].parse() {
        Ok(num) if (1..=1000).contains(&num) => num,
        _ => {
            println!("Error: Rows should be between 1 and 1000 inclusive");
            return;
        }
    };
    let cols: usize = match args[2].parse() {
        Ok(num) if (1..=18278).contains(&num) => num,
        _ => {
            println!("Error: Cols should be between 1 and 18278 inclusive");
            return;
        }
    };

    // Initialize the spreadsheet
    let mut spreadsheet = spreadsheet::initialize_spreadsheet(rows, cols);
    spreadsheet.output_disabled = false;

    if !spreadsheet.output_disabled {
        display::printer(
            spreadsheet.curr_x,
            spreadsheet.curry,
            &spreadsheet.arr,
            spreadsheet.cols,
            spreadsheet.rows,
        );
    }

    let global_start = Instant::now();
    println!("[{:.6}] (ok)", global_start.elapsed().as_secs_f64());

    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();

        if io::stdin().read_line(&mut input).is_err() {
            println!(
                "[{:.6}] (Error reading input)",
                global_start.elapsed().as_secs_f64()
            );
            break;
        }
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("q") {
            break;
        }

        let cmd_start = Instant::now();
        let mut status = 0;

        if trimmed == "disable_output" {
            spreadsheet.output_disabled = true;
            println!("[{:.6}] (Output disabled)", cmd_start.elapsed().as_secs_f64());
            continue;
        } else if trimmed == "enable_output" {
            spreadsheet.output_disabled = false;
            display::printer(
                spreadsheet.curr_x,
                spreadsheet.curry,
                &spreadsheet.arr,
                spreadsheet.cols,
                spreadsheet.rows,
            );
            println!("[{:.6}] (Output enabled)", cmd_start.elapsed().as_secs_f64());
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
            // IMPORTANT: Call the new parser function with a mutable reference
            // and the input command (formula) as a string.
            status = input_parser::parser(&mut spreadsheet, trimmed);
        }

        let elapsed = cmd_start.elapsed().as_secs_f64();
        if !spreadsheet.output_disabled {
            display::printer(
                spreadsheet.curr_x,
                spreadsheet.curry,
                &spreadsheet.arr,
                spreadsheet.cols,
                spreadsheet.rows,
            );
        }
        if status != 1 {
            println!("[{:.6}] (ok)", elapsed);
        } else {
            println!("[{:.6}] (unrecognized command)", elapsed);
        }
    }
}