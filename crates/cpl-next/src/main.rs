//! `cpl-next FILE [--raw|--canonical|--engine] [--cpp] [--check] [--json] [--trace] [--bodies]`: run FILE on
//! stdin on the engine (`--raw` and `--canonical`: on the reference interpreter, for the tests), print its C++, or only compile it; `--bodies` runs each `@fast` fn's body, not its op; `--json` prints a rejection's diagnostics as JSON lines. Exits 0 after a run, 3 after a trap, 1 when the program is rejected, 4 when
//! it is refused.
//!
//! `cpl-next --all --within MS [--trace]` reads std once and then, for each `FILE<TAB>DIR` line
//! on stdin, compiles FILE against it and runs it on `DIR/in`. It writes into DIR the rejected
//! compile's `compiled.out` (its diagnostics as `--json` prints them) and then `compiled.code`, or, for each of `raw`, `canonical` and
//! `engine`, the run's `.out`, `.err` and then `.code`, and `c++` last; then it prints `done`.
//! A run that passes MS (less its compile) ends the process, leaving it and the rest unwritten.
//! For the crate's tests and for measuring; not a shipped command.
//!
//! `cpl-next dump-ir --stage raw|owned|canonical|optimized FILE` prints FILE's IR at that stage
//! through the facade: lowering's generic fns, the linked instances after ownership, then
//! canonical and optimized IR.
use cpl_next::facade::{Session, Snapshot, Verdict};
use cpl_next::{Form, IrStage, Roots, Std};

/// The `cpl-next` binary's allocator: malloc and free took a sixth of a worker's check.
#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::io::{BufRead, Write};
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Arc;
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| {
        let at = args.iter().position(|a| a == f)?;
        args.get(at + 1)
    };
    if args.first().is_some_and(|a| a == "dump-ir") {
        return dump_ir(&args[1..]);
    }
    if has("--all") {
        let Some(ms) = value("--within").and_then(|ms| ms.parse().ok()) else {
            eprintln!("usage: cpl-next --all --within MS [--trace]");
            std::process::exit(2);
        };
        return every_case(Duration::from_millis(ms), has("--trace"), &args);
    }
    if has("--std-modules") {
        let std = judged(
            Snapshot::new(Arc::new(std_in_use()), Roots::std(), ""),
            &args,
        );
        for (m, n) in Session::open(std).std_unchecked() {
            println!("{m}\t{n}");
        }
        return;
    }
    let Some(path) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: cpl-next FILE [--raw|--canonical|--engine] [--cpp] [--check] [--json] [--trace] [--bodies] [--judge NAME=MB] | --all --within MS [--trace] | --std-modules");
        std::process::exit(2);
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(2);
        }
    };
    let mut snapshot = judged(
        Snapshot::new(Arc::new(std_in_use()), roots(path), &src),
        &args,
    );
    if has("--trace") {
        snapshot = snapshot.traced();
    }
    if has("--bodies") {
        snapshot = snapshot.with_bodies();
    }
    let session = Session::open(snapshot);
    let c = match session.program() {
        Ok(c) => c,
        Err(ds) => {
            for d in ds {
                match has("--json") {
                    true => println!("{}", cpl_json::to_string(&d.json())),
                    false => println!("{d}"),
                }
            }
            std::process::exit(code(session.verdict()));
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
    let o = match (has("--raw"), has("--canonical")) {
        (true, _) => oracle(c, Form::Raw, &input),
        (_, true) => oracle(c, Form::Canonical, &input),
        _ => c.run(&input),
    };
    print!("{}", o.output);
    eprint!("{}", said(&o));
    if o.trap.is_some() {
        std::process::exit(3);
    }
}

/// `dump-ir --stage STAGE FILE`.
fn dump_ir(args: &[String]) {
    fn usage() -> ! {
        eprintln!("usage: cpl-next dump-ir --stage raw|owned|canonical|optimized FILE");
        std::process::exit(2);
    }
    let at = args.iter().position(|a| a == "--stage");
    let Some(stage) = at
        .and_then(|i| args.get(i + 1))
        .and_then(|s| IrStage::named(s))
    else {
        usage()
    };
    let named = |i: usize| at.is_some_and(|at| i == at + 1);
    let Some((_, path)) =
        (args.iter().enumerate()).find(|&(i, a)| !a.starts_with("--") && !named(i))
    else {
        usage()
    };
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(2);
    });
    let session = Session::open(judged(
        Snapshot::new(Arc::new(std_in_use()), roots(path), &src),
        args,
    ));
    match session.ir(stage) {
        // A reader that stops early (`| head`) is not an error.
        Ok(text) => drop(std::io::stdout().write_all(text.as_bytes())),
        Err(ds) => {
            for d in ds {
                println!("{d}");
            }
            std::process::exit(code(session.verdict()));
        }
    }
}

/// Where FILE's libraries are: the `cp.toml` at or above its folder says.
fn roots(path: &str) -> Roots {
    let dir = (Path::new(path).parent()).filter(|d| !d.as_os_str().is_empty());
    Roots::from(dir.unwrap_or(Path::new(".")))
}

/// 1: the program is rejected. 4: refused, which says nothing about the program.
fn code(v: Verdict) -> i32 {
    match v {
        Verdict::Accepted => 0,
        Verdict::Rejected => 1,
        Verdict::Refused => 4,
    }
}

/// `--all`: every case on stdin compiled against one read of std.
fn every_case(limit: Duration, events: bool, args: &[String]) {
    let std = judged(
        Snapshot::new(Arc::new(std_in_use()), Roots::std(), ""),
        args,
    );
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let mut fields = line.split('\t');
        let (Some(path), Some(dir), judge) = (fields.next(), fields.next(), fields.next()) else {
            eprintln!("cpl-next --all: `{line}` is not FILE<TAB>DIR[<TAB>JUDGE=MB]");
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
                eprintln!("cpl-next --all: {line}: {e}");
                std::process::exit(2);
            }
        };
        let mut snapshot = std.with_program(&src).with_roots(roots(path));
        if let Some((name, mb)) = judge.and_then(judge_given) {
            snapshot = snapshot.with_judge(name, mb);
        }
        if events {
            snapshot = snapshot.traced();
        }
        let session = Session::open(snapshot);
        match session.program() {
            Ok(c) => every_form(c, dir, limit.saturating_sub(started.elapsed()), &input),
            Err(ds) => {
                let out: String = (ds.iter())
                    .map(|d| cpl_json::to_string(&d.json()) + "\n")
                    .collect();
                write(dir, "compiled.out", &out);
                write(dir, "compiled.code", &code(session.verdict()).to_string());
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

/// The reference interpreter's run, which only the tests ask for (`--raw`, `--canonical`,
/// `--all`): the oracle the engine and the C++ are held to, never a run of its own.
fn oracle(c: &cpl_next::facade::Program, form: Form, input: &str) -> cpl_next::Outcome {
    let (p, fs) = c.form(form);
    cpl_next_interp::run(p, fs, input)
}

/// What a run writes to stderr: its events, then its trap.
fn said(o: &cpl_next::Outcome) -> String {
    let mut s: String = o.events.iter().map(|e| format!("{e}\n")).collect();
    if let Some(t) = &o.trap {
        s += &format!("trap: {}: line {}: {}\n", t.code(), t.line, t.msg);
        if t.stack.len() > 1 {
            s += &format!("call stack: {}\n", t.stack.join(" <- "));
        }
    }
    s
}

/// Each form run in turn on one compile, as `--all` says. A watchdog gives every run `budget`,
/// the time a process of its own would have had after compiling.
fn every_form(c: &cpl_next::facade::Program, dir: &Path, budget: Duration, input: &str) {
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
        write(&format!("{name}.err"), &said(&o));
        write(
            &format!("{name}.code"),
            if o.trap.is_some() { "3" } else { "0" },
        );
        let _ = done.send(());
    }
    write("c++", &c.cpp());
    if cfg!(debug_assertions) {
        write("error-typed", &c.error_typed().join("\n"));
    }
}

/// `snapshot` built for the judge `--judge NAME=MB` names; the caller resolves it.
fn judged(snapshot: Snapshot, args: &[String]) -> Snapshot {
    let at = args.iter().position(|a| a == "--judge");
    match at
        .and_then(|i| args.get(i + 1))
        .map(|j| (j, judge_given(j)))
    {
        Some((_, Some((name, mb)))) => snapshot.with_judge(name, mb),
        Some((j, None)) => {
            eprintln!("cpl-next: `--judge {j}` is not NAME=MB");
            std::process::exit(2);
        }
        None => snapshot,
    }
}

fn judge_given(j: &str) -> Option<(&str, u64)> {
    let (name, mb) = j.split_once('=')?;
    Some((name, mb.parse().ok()?))
}

/// The std this binary was built with, or the directory `CPL_NEXT_STD_DIR` names.
fn std_in_use() -> Std {
    cpl_env::NEXT_STD_DIR
        .path()
        .map_or_else(Std::read, |dir| Std::read_dir(&dir))
}
