//! The tests' harness binary, never shipped: the reference interpreter they hold the engine
//! and the C++ to.
//!
//! `cpl-next-oracle FILE --raw|--canonical [--json] [--trace] [--bodies] [--judge NAME=MB]
//! [--std DIR]` runs FILE on stdin and stdout, streamed, on the reference interpreter, over the raw or the
//! canonical instances; it exits as `cpl-next` does.
//!
//! `cpl-next-oracle --all --within MS [--trace]` reads std once and then, for each
//! `FILE<TAB>DIR[<TAB>JUDGE=MB]` line on stdin, compiles FILE against it and runs it on
//! `DIR/in`. It writes into DIR the rejected compile's `compiled.out` (its diagnostics as
//! `--json` prints them) and then `compiled.code`, or, for each of `raw`, `canonical` and
//! `engine`, the run's `.out`, `.err` and then `.code`, then `c++` and `error-typed`; then it
//! prints `done`. A run that passes MS (less its compile) ends the process, leaving it and
//! the rest unwritten.
#[path = "../cli.rs"]
mod cli;

use cli::{code, has, judge_given, judged, said, std_in_use};
use cpl_next::facade::{Program, Session, Snapshot, World};
use cpl_next::Form;
use std::io::{BufRead, Write};
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Arc;
use std::time::Duration;

const USAGE: &str = "cpl-next-oracle FILE --raw|--canonical [--json] [--trace] [--bodies] [--judge NAME=MB] [--std DIR] | --all --within MS [--trace]";

fn main() {
    cli::on_big_stack(real_main)
}

fn real_main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if has(&args, "--all") {
        let at = args.iter().position(|a| a == "--within");
        let Some(ms) = at.and_then(|i| args.get(i + 1)?.parse().ok()) else {
            eprintln!("usage: {USAGE}");
            std::process::exit(2);
        };
        return every_case(Duration::from_millis(ms), has(&args, "--trace"), &args);
    }
    let form = match (has(&args, "--raw"), has(&args, "--canonical")) {
        (true, false) => Form::Raw,
        (false, true) => Form::Canonical,
        _ => {
            eprintln!("usage: {USAGE}");
            std::process::exit(2);
        }
    };
    let c = cli::program(&args, USAGE);
    let (p, fs) = c.form(form);
    // Output goes to stdout as the program flushes and reads, as `cpl-next`'s does.
    let input = c.streamed(Box::new(std::io::stdin()), Box::new(std::io::stdout()));
    let o = cpl_next_interp::run(p, fs, input);
    eprint!("{}", said(&c, &o));
    if o.trap.is_some() {
        std::process::exit(3);
    }
}

/// The reference interpreter's run of `c`'s `form`.
fn oracle(c: &Program, form: Form, input: &str) -> cpl_next::Outcome {
    let (p, fs) = c.form(form);
    cpl_next_interp::run(p, fs, c.input(input))
}

/// `--all`: every case on stdin compiled against one read of std.
fn every_case(limit: Duration, events: bool, args: &[String]) {
    let std = judged(
        Snapshot::new(World::Disk).with_std_lib(Arc::new(std_in_use(args))),
        args,
    );
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let mut fields = line.split('\t');
        let (Some(path), Some(dir), judge) = (fields.next(), fields.next(), fields.next()) else {
            eprintln!("cpl-next-oracle --all: `{line}` is not FILE<TAB>DIR[<TAB>JUDGE=MB]");
            std::process::exit(2);
        };
        let started = std::time::Instant::now();
        let dir = Path::new(dir);
        let (src, input) = match (
            std::fs::read_to_string(path),
            std::fs::read_to_string(dir.join("in")),
        ) {
            (Ok(src), Ok(input)) => (src, input),
            (Err(e), _) | (_, Err(e)) => {
                eprintln!("cpl-next-oracle --all: {line}: {e}");
                std::process::exit(2);
            }
        };
        let mut snapshot = std.with_file(path, &src);
        if let Some((name, mb)) = judge.and_then(judge_given) {
            snapshot = snapshot.with_judge(name, mb);
        }
        if events {
            snapshot = snapshot.traced();
        }
        let session = Session::open(snapshot);
        match session.program(path) {
            Ok(c) => every_form(&c, dir, limit.saturating_sub(started.elapsed()), &input),
            Err(ds) => {
                let out: String = (ds.iter()).map(|d| d.json() + "\n").collect();
                write(dir, "compiled.out", &out);
                write(
                    dir,
                    "compiled.code",
                    &code(session.verdict(path)).to_string(),
                );
            }
        }
        let mut stdout = std::io::stdout().lock();
        if writeln!(stdout, "done")
            .and_then(|()| stdout.flush())
            .is_err()
        {
            return;
        }
    }
}

fn write(dir: &Path, name: &str, text: &str) {
    if let Err(e) = std::fs::write(dir.join(name), text) {
        eprintln!("cpl-next: {}: {e}", dir.display());
        std::process::exit(2);
    }
}

/// Each form run in turn on one compile, as `--all` says. A watchdog gives every run `budget`,
/// the time a process of its own would have had after compiling.
fn every_form(c: &Program, dir: &Path, budget: Duration, input: &str) {
    let (done, finished) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || loop {
        match finished.recv_timeout(budget) {
            Ok(()) => {}
            Err(RecvTimeoutError::Timeout) => std::process::exit(0),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    });
    let write = |name: &str, text: &str| write(dir, name, text);
    for (name, form) in [
        ("raw", Some(Form::Raw)),
        ("canonical", Some(Form::Canonical)),
        ("engine", None),
    ] {
        let o = match form {
            Some(form) => oracle(c, form, input),
            None => c.run(input),
        };
        write(&format!("{name}.out"), &o.output);
        write(&format!("{name}.err"), &said(c, &o));
        write(
            &format!("{name}.code"),
            if o.trap.is_some() { "3" } else { "0" },
        );
        let _ = done.send(());
    }
    write("c++", &c.cpp());
    write("error-typed", &c.error_typed().join("\n"));
}
