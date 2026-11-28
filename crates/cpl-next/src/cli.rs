//! What `cpl-next` and the tests' `cpl-next-oracle` read from their arguments alike: the file,
//! the judge, the std, the compile and what a run writes to stderr.
use cpl_next::facade::{Program, Session, Snapshot, Verdict, World};
use cpl_next::{Form, Std};
use cpl_next_ir::build::Judge;
use cpl_next_ir::run::Limits;
use std::path::Path;
use std::sync::Arc;

/// The binaries' allocator: malloc and free took a sixth of a worker's check.
#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Runs `f` on a thread whose stack is the run's (spec 9.6): `Limits::stack`, the judge's, the
/// number the C++ target and S19 report. The interpreters grow theirs past it as a call needs
/// (`stacker`), so the run's own count is what runs out (R0601). A stack the machine will not
/// give is an error saying so.
pub fn on_run_stack<T: Send>(limits: Limits, f: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|s| {
        let spawned = (std::thread::Builder::new())
            .stack_size(usize::try_from(limits.stack).unwrap_or(usize::MAX))
            .spawn_scoped(s, f);
        match spawned {
            Ok(t) => t.join().unwrap_or_else(|_| std::process::exit(101)),
            Err(e) => {
                eprintln!(
                    "cpl-next: this machine will not give the run the judge's {} MB stack: {e}",
                    limits.stack >> 20
                );
                std::process::exit(2);
            }
        }
    })
}

/// The limits of a run on the judge `--judge NAME=MB` names, or the generic judge's.
pub fn limits(args: &[String]) -> Limits {
    let at = args.iter().position(|a| a == "--judge");
    let given = at
        .and_then(|i| args.get(i + 1))
        .and_then(|j| judge_given(j));
    limits_on(given)
}

/// The limits of a run on the judge `NAME=MB` gives, or the generic judge's.
pub fn limits_on(given: Option<(&str, u64)>) -> Limits {
    let judge = given.map(|(name, memory_mb)| Judge {
        name: name.into(),
        memory_mb,
    });
    Limits::of(judge.as_ref())
}

pub fn has(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// FILE compiled as `--trace`, `--bodies`, `--judge` and `--std` say. A compile that rejects
/// or refuses it prints its diagnostics (as JSON lines with `--json`) and exits.
pub fn program(args: &[String], usage: &str) -> Arc<Program> {
    let Some(path) = file(args) else {
        eprintln!("usage: {usage}");
        std::process::exit(2);
    };
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(2);
    });
    let mut snapshot = judged(
        (Snapshot::new(World::Disk).with_std_lib(Arc::new(std_in_use(args)))).with_file(path, &src),
        args,
    );
    if has(args, "--trace") {
        snapshot = snapshot.traced();
    }
    if has(args, "--bodies") {
        snapshot = snapshot.with_bodies();
    }
    let session = Session::open(snapshot);
    match session.program(path) {
        Ok(c) => c,
        Err(ds) => {
            for d in ds.iter() {
                match has(args, "--json") {
                    true => println!("{}", d.json()),
                    false => println!("{d}"),
                }
            }
            std::process::exit(code(session.verdict(path)));
        }
    }
}

/// 1: the program is rejected. 4: refused, which says nothing about the program.
pub fn code(v: Verdict) -> i32 {
    match v {
        Verdict::Accepted => 0,
        Verdict::Rejected => 1,
        Verdict::Refused => 4,
    }
}

/// What a run writes to stderr: its events, then its trap.
pub fn said(c: &Program, o: &cpl_next::Outcome) -> String {
    let funcs = c.form(Form::Raw).1;
    let mut s: String = o.events.iter().map(|e| e.line(funcs) + "\n").collect();
    if let Some(t) = &o.trap {
        s += &c.said(t);
    }
    s
}

/// The flags that take the next argument as their value.
const VALUED: [&str; 4] = ["--std", "--judge", "--within", "--stage"];

/// The one argument that is neither a flag nor a flag's value.
pub fn file(args: &[String]) -> Option<&String> {
    let value = |i: usize| i > 0 && VALUED.contains(&args[i - 1].as_str());
    (args.iter().enumerate()).find_map(|(i, a)| (!a.starts_with("--") && !value(i)).then_some(a))
}

/// `snapshot` built for the judge `--judge NAME=MB` names; the caller resolves it.
pub fn judged(snapshot: Snapshot, args: &[String]) -> Snapshot {
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

pub fn judge_given(j: &str) -> Option<(&str, u64)> {
    let (name, mb) = j.split_once('=')?;
    Some((name, mb.parse().ok()?))
}

/// The std this binary was built with, or the directory `--std DIR` names.
pub fn std_in_use(args: &[String]) -> Std {
    let at = args.iter().position(|a| a == "--std");
    match at.and_then(|i| args.get(i + 1)) {
        Some(dir) => Std::read_dir(Path::new(dir)),
        None => Std::read(),
    }
}
