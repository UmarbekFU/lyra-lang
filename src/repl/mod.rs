pub mod highlighter;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::eval;
use crate::eval::env::Env;
use crate::lexer;
use crate::parser;
use crate::types::env::TypeEnv;
use crate::types::infer::Inferencer;
use crate::types::TypeVarGen;
use crate::stdlib;

use highlighter::LyraHelper;

pub fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    let config = rustyline::Config::builder()
        .auto_add_history(true)
        .build();

    let helper = LyraHelper;
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(helper));

    // Load history
    let history_path = dirs_next().unwrap_or_default();
    let _ = rl.load_history(&history_path);

    // Persistent environments
    let mut type_env = TypeEnv::new();
    let runtime_env = Env::new();
    let mut gen = TypeVarGen::new();
    let mut inferencer = Inferencer::new();

    stdlib::register_stdlib(&mut type_env, &runtime_env, &mut gen);

    println!("\x1b[1;35mLyra\x1b[0m v10.0 — A functional programming language");
    println!("Type \x1b[1m:help\x1b[0m for help, \x1b[1m:quit\x1b[0m to exit\n");

    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() {
            "\x1b[1;35mlyra>\x1b[0m "
        } else {
            "\x1b[1;35m ...>\x1b[0m "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let line = line.trim_end();

                if buffer.is_empty() && line.is_empty() {
                    continue;
                }

                // REPL commands (only on first line)
                if buffer.is_empty() {
                    match line {
                        ":quit" | ":q" => break,
                        ":help" | ":h" => {
                            print_help();
                            continue;
                        }
                        ":env" => {
                            println!("  (type environment display not yet implemented)");
                            continue;
                        }
                        _ if line.starts_with(":type ") => {
                            let expr_src = &line[6..];
                            match infer_type(expr_src, &type_env, &mut inferencer) {
                                Ok(ty) => println!("  \x1b[36m: {}\x1b[0m", ty),
                                Err(e) => eprintln!("{}", e.render(expr_src, "<repl>")),
                            }
                            continue;
                        }
                        _ if line.starts_with(":load ") => {
                            let path = line[6..].trim();
                            match std::fs::read_to_string(path) {
                                Ok(source) => {
                                    match eval_line(
                                        &source,
                                        &mut type_env,
                                        &runtime_env,
                                        &mut inferencer,
                                    ) {
                                        Ok(_) => {
                                            println!(
                                                "  \x1b[32mLoaded {}\x1b[0m",
                                                path
                                            );
                                        }
                                        Err(e) => {
                                            eprintln!("{}", e.render(&source, path));
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("\x1b[1;31merror\x1b[0m: {}", e);
                                }
                            }
                            continue;
                        }
                        _ => {}
                    }
                }

                // Accumulate multi-line input
                if !buffer.is_empty() {
                    buffer.push('\n');
                }
                buffer.push_str(line);

                // Check if input looks complete
                if !is_complete(&buffer) {
                    continue;
                }

                let source = buffer.clone();
                buffer.clear();

                // Normal pipeline: lex -> parse -> typecheck -> eval
                match eval_line(&source, &mut type_env, &runtime_env, &mut inferencer) {
                    Ok(Some((value, ty))) => {
                        println!("  \x1b[1m{}\x1b[0m \x1b[36m: {}\x1b[0m", value, ty);
                    }
                    Ok(None) => {} // declaration bound, no output
                    Err(e) => {
                        eprintln!("{}", e.render(&source, "<repl>"));
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                if !buffer.is_empty() {
                    buffer.clear();
                    println!("  \x1b[33m(input cancelled)\x1b[0m");
                } else {
                    println!("^C");
                }
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
    println!("Goodbye!");
    Ok(())
}

/// Check if a multi-line input looks complete (no unclosed delimiters, etc.)
fn is_complete(source: &str) -> bool {
    // Count unclosed delimiters
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut in_string = false;
    let mut prev_char = '\0';

    for ch in source.chars() {
        if in_string {
            if ch == '"' && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            _ => {}
        }
        prev_char = ch;
    }

    if paren_depth > 0 || bracket_depth > 0 || brace_depth > 0 || in_string {
        return false;
    }

    // Check for trailing tokens that expect continuation
    let trimmed = source.trim_end();
    if trimmed.ends_with("->")
        || trimmed.ends_with("with")
        || trimmed.ends_with('=')
        || trimmed.ends_with("then")
        || trimmed.ends_with("else")
        || trimmed.ends_with("in")
        || trimmed.ends_with('|')
    {
        return false;
    }

    true
}

fn eval_line(
    source: &str,
    type_env: &mut TypeEnv,
    runtime_env: &Env,
    inferencer: &mut Inferencer,
) -> Result<Option<(eval::value::Value, crate::types::MonoType)>, crate::error::LyraError> {
    let tokens = lexer::tokenize(source).map_err(|errs| errs[0].clone())?;
    let decls = parser::parse(tokens)?;

    let mut last_result = None;

    for decl in &decls {
        let ty = inferencer.infer_decl(type_env, decl)?;
        let val = eval::eval_decl(runtime_env, decl)?;

        if let (Some(v), Some(t)) = (val, ty) {
            last_result = Some((v, t));
        }
    }

    Ok(last_result)
}

fn infer_type(
    source: &str,
    type_env: &TypeEnv,
    inferencer: &mut Inferencer,
) -> Result<crate::types::MonoType, crate::error::LyraError> {
    let tokens = lexer::tokenize(source).map_err(|errs| errs[0].clone())?;
    let decls = parser::parse(tokens)?;

    if let Some(decl) = decls.first() {
        match decl {
            crate::ast::Decl::Expr(expr) => {
                let (_, ty) = inferencer.infer(type_env, expr)?;
                Ok(ty)
            }
            _ => {
                let mut env = type_env.clone();
                let ty = inferencer.infer_decl(&mut env, decl)?;
                ty.ok_or_else(|| crate::error::LyraError::RuntimeError {
                    message: "no type to display".to_string(),
                    span: crate::span::Span::default(),
                })
            }
        }
    } else {
        Err(crate::error::LyraError::RuntimeError {
            message: "empty input".to_string(),
            span: crate::span::Span::default(),
        })
    }
}

fn print_help() {
    println!("\x1b[1mLyra REPL Commands:\x1b[0m");
    println!("  :help, :h          Show this help message");
    println!("  :quit, :q          Exit the REPL");
    println!("  :type <expr>       Show the type of an expression");
    println!("  :load <file>       Load and evaluate a .lyra file");
    println!("  :env               Show the type environment");
    println!();
    println!("\x1b[1mLanguage Features:\x1b[0m");
    println!("  let x = 42                              Bind a value");
    println!("  let rec f = fn (n) -> ...               Recursive function");
    println!("  fn (x, y) -> x + y                      Lambda function");
    println!("  if x > 0 then x else -x                 Conditional");
    println!("  match x with | 0 -> a | n -> b          Pattern matching");
    println!("  type Option a = Some a | None            Algebraic data types");
    println!("  [1, 2, 3] |> map(fn (x) -> x * 2)      Pipe operator");
    println!("  1 :: [2, 3]                              Cons operator");
    println!("  \"hello {{name}}\"                          String interpolation");
    println!("  {{ name: \"Alice\", age: 30 }}               Record types");
    println!("  person.name                              Field access");
    println!("  import \"utils\"                            Module imports");
    println!();
    println!("\x1b[1mBuilt-in Functions:\x1b[0m");
    println!("  print, println, to_string");
    println!("  map, filter, fold, zip, sort");
    println!("  head, tail, length, reverse, append, range, nth");
    println!("  abs, min, max, pow");
    println!("  str_length, str_concat, str_split, str_chars, str_contains");
    println!("  float_of_int, int_of_float");
    println!();
    println!("\x1b[1mMulti-line Input:\x1b[0m");
    println!("  Unclosed parens/brackets or trailing -> automatically continue to next line");
}

fn dirs_next() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".lyra_history"))
}
