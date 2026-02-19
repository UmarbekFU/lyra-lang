pub mod ast;
pub mod compiler;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod span;
pub mod stdlib;
pub mod types;
pub mod vm;

use std::collections::HashSet;
use std::path::Path;

use error::LyraError;
use eval::env::Env;
use types::env::TypeEnv;
use types::infer::Inferencer;
use types::TypeVarGen;

/// Resolve an import path relative to the current file.
fn resolve_import(current_file: &str, import_path: &str) -> String {
    let base = Path::new(current_file)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let mut resolved = base.join(import_path);
    if resolved.extension().is_none() {
        resolved.set_extension("lyra");
    }
    resolved.to_string_lossy().to_string()
}

/// Run a Lyra source file using the tree-walking interpreter.
pub fn run_file(source: &str, filename: &str) -> Result<(), LyraError> {
    let mut imported = HashSet::new();
    run_file_inner(source, filename, &mut imported)
}

fn run_file_inner(
    source: &str,
    filename: &str,
    imported: &mut HashSet<String>,
) -> Result<(), LyraError> {
    let tokens = lexer::tokenize(source).map_err(|errs| errs[0].clone())?;
    let decls = parser::parse(tokens)?;

    let mut type_env = TypeEnv::new();
    let runtime_env = Env::new();
    let mut gen = TypeVarGen::new();
    let mut inferencer = Inferencer::new();

    stdlib::register_stdlib(&mut type_env, &runtime_env, &mut gen);

    for decl in &decls {
        // Handle imports by loading the file and evaluating it
        if let ast::Decl::Import { path, span } = decl {
            let resolved = resolve_import(filename, path);
            if imported.contains(&resolved) {
                continue; // already imported
            }
            imported.insert(resolved.clone());
            let import_source = std::fs::read_to_string(&resolved).map_err(|e| {
                LyraError::RuntimeError {
                    message: format!("cannot import \"{}\": {}", path, e),
                    span: *span,
                }
            })?;
            // Parse and evaluate the imported file in the same environments
            let import_tokens =
                lexer::tokenize(&import_source).map_err(|errs| errs[0].clone())?;
            let import_decls = parser::parse(import_tokens)?;
            for import_decl in &import_decls {
                if let Err(e) = inferencer.infer_decl(&mut type_env, import_decl) {
                    eprintln!("{}", e.render(&import_source, &resolved));
                    return Err(e);
                }
                if let Err(e) = eval::eval_decl(&runtime_env, import_decl) {
                    eprintln!("{}", e.render(&import_source, &resolved));
                    return Err(e);
                }
            }
            continue;
        }

        if let Err(e) = inferencer.infer_decl(&mut type_env, decl) {
            eprintln!("{}", e.render(source, filename));
            return Err(e);
        }
        if let Err(e) = eval::eval_decl(&runtime_env, decl) {
            eprintln!("{}", e.render(source, filename));
            return Err(e);
        }
    }

    Ok(())
}

/// Run a Lyra source file using the bytecode compiler + VM.
pub fn run_file_vm(source: &str, filename: &str) -> Result<(), LyraError> {
    let tokens = lexer::tokenize(source).map_err(|errs| errs[0].clone())?;
    let mut decls = parser::parse(tokens)?;

    // Resolve imports: inline imported file declarations
    let mut imported = HashSet::new();
    resolve_imports(&mut decls, filename, &mut imported)?;

    // Type check
    let mut type_env = TypeEnv::new();
    let runtime_env = Env::new();
    let mut gen = TypeVarGen::new();
    let mut inferencer = Inferencer::new();

    stdlib::register_stdlib(&mut type_env, &runtime_env, &mut gen);

    for decl in &decls {
        if let Err(e) = inferencer.infer_decl(&mut type_env, decl) {
            eprintln!("{}", e.render(source, filename));
            return Err(e);
        }
    }

    // Compile to bytecode
    let proto = compiler::compile(&decls).map_err(|msg| LyraError::RuntimeError {
        message: msg,
        span: span::Span::default(),
    })?;

    // Execute on VM
    let mut machine = vm::VM::new();
    stdlib::register_vm_stdlib(&mut machine);
    if let Err(e) = machine.run(proto) {
        eprintln!("{}", e.render(source, filename));
        return Err(e);
    }

    Ok(())
}

/// Inline import declarations by replacing them with the imported file's declarations.
fn resolve_imports(
    decls: &mut Vec<ast::Decl>,
    current_file: &str,
    imported: &mut HashSet<String>,
) -> Result<(), LyraError> {
    let mut i = 0;
    while i < decls.len() {
        if let ast::Decl::Import { path, span } = &decls[i] {
            let resolved = resolve_import(current_file, path);
            let span = *span;
            if imported.contains(&resolved) {
                decls.remove(i);
                continue;
            }
            imported.insert(resolved.clone());
            let import_source =
                std::fs::read_to_string(&resolved).map_err(|e| LyraError::RuntimeError {
                    message: format!("cannot import \"{}\": {}", path, e),
                    span,
                })?;
            let import_tokens =
                lexer::tokenize(&import_source).map_err(|errs| errs[0].clone())?;
            let mut import_decls = parser::parse(import_tokens)?;
            // Recursively resolve imports in the imported file
            resolve_imports(&mut import_decls, &resolved, imported)?;
            // Replace the Import decl with the imported declarations
            decls.remove(i);
            for (j, d) in import_decls.into_iter().enumerate() {
                decls.insert(i + j, d);
            }
            // Don't increment i — process newly inserted decls
        } else {
            i += 1;
        }
    }
    Ok(())
}
