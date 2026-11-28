//! `cpl-next FILE [--engine] [--cpp] [--check] [--json] [--trace] [--bodies]`: run FILE on
//! stdin on the engine, print its C++, or only compile it; `--bodies` runs each `@fast` fn's body, not its op; `--json` prints a rejection's diagnostics as JSON lines. Exits 0 after a run, 3 after a trap, 1 when the program is rejected, 4 when
//! it is refused.
//!
//! `cpl-next dump-ir --stage raw|owned|canonical|optimized FILE` prints FILE's IR at that stage
//! through the facade: lowering's generic fns, the linked instances after ownership, then
//! canonical and optimized IR.
//!
//! `cpl-next --dump-own FILE` (`dump-ir --stage borrows`) prints, for each binding ownership's
//! reporting walk binds, the places a write to which conflicts with it (`frozen`) and the places
//! it lends (`lent`), even when ownership rejects the program.
//!
//! `cpl-next dump-ir --stage instances FILE` prints each instance link made, the instance whose
//! body named it (`by #j`, or `root`) and the set read that held it, for a set bug.
//!
//! The reference interpreter, the oracle the tests hold the engine and the C++ to, is not in
//! this binary: the tests run it through `cpl-next-oracle`.
mod cli;

use cli::{code, file, has, judged, said, std_in_use};
use cpl_next::facade::{Session, Snapshot, World};
use cpl_next::IrStage;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    cli::on_run_stack(cli::limits(&args), real_main)
}

fn real_main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(f) = (args.iter()).find(|a| a.starts_with("--") && !FLAGS.contains(&a.as_str())) {
        eprintln!("cpl-next: no flag `{f}`; usage: {USAGE}");
        std::process::exit(2);
    }
    let has = |f: &str| has(&args, f);
    if args.first().is_some_and(|a| a == "dump-ir") {
        return dump_ir(&args[1..]);
    }
    if has("--dump-own") {
        return dump_own(&args);
    }
    if has("--std-modules") {
        let std = judged(std_in_use(Snapshot::new(World::Closed), &args), &args);
        // `--std-modules std.x` shows the diagnostics of the modules it begins; alone, each
        // module's count.
        let at = args.iter().position(|a| a == "--std-modules");
        let only = at
            .and_then(|i| args.get(i + 1))
            .filter(|a| !a.starts_with("--"));
        for (m, ds) in Session::open(std).std_unchecked() {
            match only {
                None => println!("{m}\t{}", ds.len()),
                Some(o) if m.starts_with(o.as_str()) => {
                    ds.iter().for_each(|d| println!("{m}: {d}"))
                }
                Some(_) => {}
            }
        }
        return;
    }
    let c = cli::program(&args, USAGE);
    if has("--check") {
        return;
    }
    if has("--cpp") {
        print!("{}", c.cpp());
        return;
    }
    // Output goes to stdout as the program flushes and reads, so an interactor sees each query.
    let o = c.run_on(c.streamed(Box::new(std::io::stdin()), Box::new(std::io::stdout())));
    let c = &*c;
    eprint!("{}", said(c, &o));
    if o.trap.is_some() {
        std::process::exit(3);
    }
}

/// A session over FILE as it is on disk, with `--std` and `--judge` applied.
fn disk_session(args: &[String], path: &str) -> Session {
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(2);
    });
    Session::open(judged(
        std_in_use(Snapshot::new(World::Disk), args).with_file(path, &src),
        args,
    ))
}

/// `--dump-own FILE`: each binding's frozen and lent places, from ownership's own walk.
fn dump_own(args: &[String]) {
    let Some(path) = file(args) else {
        eprintln!("usage: {USAGE}");
        std::process::exit(2);
    };
    let session = disk_session(args, path);
    match session.ir(path, IrStage::Borrows) {
        Ok(text) => drop(std::io::stdout().write_all(text.as_bytes())),
        Err(ds) => {
            for d in ds.iter() {
                println!("{d}");
            }
            std::process::exit(code(session.verdict(path)));
        }
    }
}

/// `dump-ir --stage STAGE FILE`.
fn dump_ir(args: &[String]) {
    let at = args.iter().position(|a| a == "--stage");
    let stage = at
        .and_then(|i| args.get(i + 1))
        .and_then(|s| IrStage::named(s));
    let (Some(stage), Some(path)) = (stage, file(args)) else {
        eprintln!(
            "usage: cpl-next dump-ir --stage raw|owned|canonical|optimized|borrows|instances FILE"
        );
        std::process::exit(2);
    };
    let session = disk_session(args, path);
    match session.ir(path, stage) {
        // A reader that stops early (`| head`) is not an error.
        Ok(text) => drop(std::io::stdout().write_all(text.as_bytes())),
        Err(ds) => {
            for d in ds.iter() {
                println!("{d}");
            }
            std::process::exit(code(session.verdict(path)));
        }
    }
}

const USAGE: &str = "cpl-next FILE [--engine] [--cpp] [--check] [--json] [--trace] [--bodies] [--judge NAME=MB] [--std DIR] | --std-modules [MODULE] | --dump-own FILE | dump-ir --stage STAGE FILE";

/// The flags `cpl-next` takes; any other is refused, never ignored.
const FLAGS: [&str; 11] = [
    "--engine",
    "--cpp",
    "--check",
    "--json",
    "--trace",
    "--bodies",
    "--judge",
    "--std",
    "--std-modules",
    "--stage",
    "--dump-own",
];
