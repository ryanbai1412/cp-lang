//! `cpl-next FILE [--raw|--engine] [--cpp] [--check] [--trace]`: run FILE on stdin, print its C++, or
//! only compile it. `--all DIR --within MS` compiles once and writes, for each of `raw`,
//! `canonical` and `engine`, the run's `.out`, `.err` and then `.code` into DIR, and `c++` last;
//! a run that passes MS (less the compile) ends the process, leaving it and the rest unwritten.
//! Exits 0 after a run, 3 after a trap, 1 when the program is rejected, 4 when it is refused.
//! For the crate's tests and for measuring; not a shipped command.
use cpl_next::{compile, Form, Roots};
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

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
    let started = std::time::Instant::now();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| {
        let at = args.iter().position(|a| a == f)?;
        args.get(at + 1)
    };
    let all = value("--all").zip(value("--within").and_then(|ms| ms.parse().ok()));
    let values: Vec<&String> = [value("--all"), value("--within")]
        .into_iter()
        .flatten()
        .collect();
    let Some(path) = args
        .iter()
        .find(|a| !a.starts_with("--") && !values.iter().any(|v| std::ptr::eq(*v, *a)))
    else {
        eprintln!("usage: cpl-next FILE [--raw|--engine] [--cpp] [--check] [--trace] [--all DIR --within MS]");
        std::process::exit(2);
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(2);
        }
    };
    let dir = (Path::new(path).parent()).filter(|d| !d.as_os_str().is_empty());
    let roots = Roots::from(dir.unwrap_or(Path::new(".")));
    let c = match compile(&src, &roots, has("--trace")) {
        Ok(c) => c,
        Err(ds) => {
            for d in &ds {
                println!("{d}");
            }
            // 1: the program is rejected. 4: refused, which says nothing about the program. A
            // syntax error rejects it whatever a later layer refused.
            let syntax = |d: &cpl_next::Diag| d.stage == cpl_next::Stage::Syntax && !d.is_refusal();
            let refused = ds.iter().any(cpl_next::Diag::is_refusal);
            std::process::exit(if refused && !ds.iter().any(syntax) {
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
    if let Some((dir, ms)) = all {
        let budget = std::time::Duration::from_millis(ms).saturating_sub(started.elapsed());
        every_form(&c, Path::new(dir), budget, &input);
        return;
    }
    let o = c.run(
        if has("--raw") {
            Form::Raw
        } else if has("--engine") {
            Form::Engine
        } else {
            Form::Canonical
        },
        &input,
    );
    print!("{}", o.output);
    eprint!("{}", said(&o));
    if o.trap.is_some() {
        std::process::exit(3);
    }
}

/// What a run writes to stderr: its events, then its trap.
fn said(o: &cpl_next::Outcome) -> String {
    let mut s: String = o.events.iter().map(|e| format!("{e}\n")).collect();
    if let Some(t) = &o.trap {
        s += &format!("trap: {}: line {}: {}\n", t.code(), t.line, t.msg);
    }
    s
}

/// Each form run in turn on one compile, as `--all` says. A watchdog gives every run `budget`,
/// the time a process of its own would have had after compiling.
fn every_form(c: &cpl_next::Compiled, dir: &Path, budget: Duration, input: &str) {
    let (done, finished) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || loop {
        match finished.recv_timeout(budget) {
            Ok(()) => {}
            Err(RecvTimeoutError::Timeout) => std::process::exit(0),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    });
    let write = |name: &str, text: &str| {
        if let Err(e) = std::fs::write(dir.join(name), text) {
            eprintln!("cpl-next: {}: {e}", dir.display());
            std::process::exit(2);
        }
    };
    for (name, form) in [
        ("raw", Form::Raw),
        ("canonical", Form::Canonical),
        ("engine", Form::Engine),
    ] {
        let o = c.run(form, input);
        write(&format!("{name}.out"), &o.output);
        write(&format!("{name}.err"), &said(&o));
        write(
            &format!("{name}.code"),
            if o.trap.is_some() { "3" } else { "0" },
        );
        let _ = done.send(());
    }
    write("c++", &c.cpp());
}
