//! Every input a program reads is one `cpl stress` can generate
//! (`debugging/R11` in `handbook/guidelines/debugging-checklist.md`): each
//! corpus problem's spec, each saved-trace program and each conformance
//! program that builds gets an input spec. There is no list of exceptions: a
//! program stress can't generate fails here until the generator learns it.

use std::path::{Path, PathBuf};

use cpl_stress::spec::InputSpec;

#[path = "../../cpl-cli/tests/support/shards.rs"]
mod shards;

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|x| x == "cp") {
            out.push(path);
        }
    }
}

const SHARDS: usize = 4;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The corpus specs, conformance programs and saved-trace programs, sorted.
fn programs(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in [
        "corpus/problems",
        "spec/conformance",
        "crates/cpl-cli/tests/programs",
    ] {
        walk(&root.join(dir), &mut files);
    }
    files.retain(|f| !f.starts_with(root.join("corpus")) || f.ends_with("spec.cp"));
    files.sort();
    files
}

/// A program's path under the repo root, which picks its shard.
fn key(path: &Path) -> String {
    path.strip_prefix(root()).unwrap().display().to_string()
}

fn keys() -> Vec<String> {
    programs(&root()).iter().map(|p| key(p)).collect()
}

/// `path: why` for each program of shard `k` whose input stress can't generate.
fn gaps(root: &Path, k: usize) -> Vec<String> {
    let mut gaps = Vec::new();
    for file in shards::mine(programs(root), |p| key(p), k, SHARDS) {
        let src = std::fs::read_to_string(&file).unwrap();
        let builds = src
            .lines()
            .find_map(|l| l.strip_prefix("// expect: "))
            .is_none_or(|expect| expect.starts_with("ok") || expect.starts_with("trap"));
        if !builds || !src.contains("read") {
            continue;
        }
        if let Err(error) = InputSpec::from_source(&src) {
            let rel = file.strip_prefix(root).unwrap().display().to_string();
            gaps.push(format!("{rel}: {}", error.message));
        }
    }
    gaps
}

fn every_readable_input_can_be_generated(k: usize) {
    let found = gaps(&root(), k);
    assert!(
        found.is_empty(),
        "stress can't generate these inputs; teach the generator the shape:\n{}",
        found.join("\n")
    );
}

shards::shard_tests!(
    every_readable_input_can_be_generated,
    keys(),
    the_readable_shards_cover_every_program,
    [
        every_readable_input_can_be_generated_0 = 0,
        every_readable_input_can_be_generated_1 = 1,
        every_readable_input_can_be_generated_2 = 2,
        every_readable_input_can_be_generated_3 = 3,
    ]
);

/// `name` filled in each way its generics can be: by a domain, `Int`, or a
/// modulus.
fn filled(name: &str, arity: usize) -> Vec<String> {
    let mut out = vec![Vec::new()];
    for at in 0..arity {
        let domain = if at == 1 { "E" } else { "V" };
        out = out
            .into_iter()
            .flat_map(|args: Vec<&str>| {
                [domain, "Int", "998244353"].map(|arg| [args.clone(), vec![arg]].concat())
            })
            .collect();
    }
    out.into_iter()
        .map(|args| match args.is_empty() {
            true => name.to_string(),
            false => format!("{name}<{}>", args.join(", ")),
        })
        .collect()
}

/// A program reading one `ty` from `std.<module>`, when the checker and
/// the lowering accept that read.
fn reads(
    db: &mut cpl_facade::Session,
    module: &str,
    name: &str,
    ty: &str,
) -> Option<(String, cpl_facade::Program)> {
    let src = format!(
        "use std.{module}.{{{name}}}\nlet (n, m): (1..=8, 1..=8) = read()\n\
         domain V = 1..=n\ndomain E = 0..m\nlet x: {ty} = read()\nprint(n)\n"
    );
    *db = db.with_file("s.cp", src.as_str());
    let error = |d: &cpl_diag::Diagnostic| d.severity == cpl_diag::Severity::Error;
    if (db.diagnostics("s.cp", cpl_facade::Layer::Built).iter()).any(error) {
        return None;
    }
    let program = db.program("s.cp", &cpl_facade::Job::program()).ok()?;
    Some((src, program))
}

/// `module.ty: why` when stress can't make an input the program reads.
fn std_gap(module: &str, ty: &str, src: &str, program: &cpl_facade::Program) -> Option<String> {
    let spec = match InputSpec::from_source(src) {
        Ok(spec) => spec,
        Err(error) => return Some(format!("{module}.{ty}: {}", error.message)),
    };
    let size = cpl_stress::Size::Small(8);
    let cases: Vec<_> = (0..8)
        .filter_map(|seed| cpl_stress::generate(&spec, &mut cpl_stress::Rng::new(seed), size))
        .collect();
    if cases.is_empty() {
        return Some(format!("{module}.{ty}: no case from 8 seeds"));
    }
    for case in cases {
        let input = case.render();
        let mut out = Vec::new();
        if let Err(trap) = cpl_facade::run_to_end(
            program,
            cpl_facade::RunInput::fixed(input.as_bytes()),
            &mut out,
        ) {
            return Some(format!(
                "{module}.{ty}: {} reading {input:?}",
                trap.message()
            ));
        }
    }
    None
}

/// Every std type `read()` accepts is one stress generates, and the program
/// reads what it makes (`stdlib.md`, "Stress can generate every std input
/// type"). The types are the std names the checker knows, each filled every
/// way; the checker and the lowering decide which ones are read.
#[test]
fn stress_generates_every_std_input_type() {
    let names: Vec<(&str, &str, usize)> = cpl_facade::Std
        .names()
        .iter()
        .filter_map(|&(name, module)| Some((module, name, cpl_facade::today::type_arity(name)?)))
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let (mut read, mut gaps) = (Vec::new(), Vec::new());
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                std::thread::Builder::new()
                    .stack_size(1 << 26)
                    .spawn_scoped(scope, || {
                        let (mut read, mut gaps) = (Vec::new(), Vec::new());
                        let mut db = cpl_facade::Session::default();
                        while let Some(&(module, name, arity)) =
                            names.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                        {
                            let Some((ty, (src, program))) =
                                filled(name, arity).into_iter().find_map(|ty| {
                                    Some((ty.clone(), reads(&mut db, module, name, &ty)?))
                                })
                            else {
                                continue;
                            };
                            gaps.extend(std_gap(module, &ty, &src, &program));
                            read.push(format!("{module}.{ty}"));
                        }
                        (read, gaps)
                    })
                    .unwrap()
            })
            .collect();
        for worker in workers {
            let (r, g) = worker.join().unwrap();
            read.extend(r);
            gaps.extend(g);
        }
    });
    gaps.sort();
    assert!(!read.is_empty(), "no std type is read");
    assert!(
        gaps.is_empty(),
        "stress can't generate these std types `read()` accepts; teach the generator:\n{}",
        gaps.join("\n")
    );
}
