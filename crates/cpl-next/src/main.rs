//! `cpl-next FILE [--raw] [--cpp] [--check] [--trace]`: run FILE on stdin, print its C++, or
//! only compile it.
//! Exits 0 after a run, 3 after a trap, 1 when the program is rejected, 4 when it is refused.
//! For the crate's tests and for measuring; not a shipped command.
use cpl_next::{compile, Form};

/// Spec 9.6: the program runs on a thread with a 1 GB stack. The interpreter's frames are
/// larger than the compiled program's, so it gets more, to never be the first to run out.
fn main() {
    match std::thread::Builder::new()
        .stack_size(16 << 30)
        .spawn(real_main)
    {
        Ok(t) => {
            if t.join().is_err() {
                std::process::exit(101);
            }
        }
        Err(e) => {
            eprintln!("cpl-next: no thread for the program: {e}");
            std::process::exit(2);
        }
    }
}

fn real_main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let Some(path) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: cpl-next FILE [--raw] [--cpp] [--check] [--trace]");
        std::process::exit(2);
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(2);
        }
    };
    let c = match compile(&src, has("--trace")) {
        Ok(c) => c,
        Err(ds) => {
            for d in &ds {
                println!("{}: {}", d.stage.name(), d.text);
            }
            // 1: the program is rejected. 4: refused, which says nothing about the program.
            std::process::exit(if ds.iter().any(cpl_next::Diag::is_refusal) {
                4
            } else {
                1
            });
        }
    };
    if has("--check") {
        return;
    }
    if has("--cpp") {
        print!("{}", c.cpp());
        return;
    }
    let mut input = String::new();
    if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input) {
        eprintln!("cpl-next: stdin: {e}");
        std::process::exit(2);
    }
    let o = c.run(
        if has("--raw") {
            Form::Raw
        } else {
            Form::Canonical
        },
        &input,
    );
    print!("{}", o.output);
    for e in &o.events {
        eprintln!("{e}");
    }
    if let Some(t) = o.trap {
        eprintln!("trap: {}: line {}: {}", t.code(), t.line, t.msg);
        std::process::exit(3);
    }
}
