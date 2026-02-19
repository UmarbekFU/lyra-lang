use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for --vm flag
    let use_vm = args.iter().any(|a| a == "--vm");
    let file_args: Vec<&String> = args.iter().skip(1).filter(|a| *a != "--vm").collect();

    match file_args.len() {
        0 => {
            // No arguments: launch REPL
            if let Err(e) = lyra::repl::run_repl() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        1 => {
            // One argument: execute file
            let path = file_args[0];
            match fs::read_to_string(path) {
                Ok(source) => {
                    let result = if use_vm {
                        lyra::run_file_vm(&source, path)
                    } else {
                        lyra::run_file(&source, path)
                    };
                    if let Err(e) = result {
                        // Errors from type-check/eval are already printed by run_file/run_file_vm
                        // but lexer/parser errors may not be, so print them too
                        eprintln!("{}", e.render(&source, path));
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading {}: {}", path, e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Usage: lyra [--vm] [file.lyra]");
            process::exit(1);
        }
    }
}
