//! Every conformance case under `spec/conformance`, and every program in `tests/programs`
//! (the same header format), against the result its header states. The header comes from
//! the spec, not from what this crate or today's compiler prints.
//!
//! A case that runs is run three ways on its `// input:`: the reference interpreter on the
//! linked instances, the same on canonical IR, and the emitted C++ under ASan and UBSan.
//! All three give the same stdout, exit status and events, within 10 seconds each.
//!
//! Each case ends one of three ways, and the two lists beside this file say which:
//! - it passes: its id is in `tests/passing.txt`;
//! - it is wrong (accepted, rejected or answered against its header, a crash, a split
//!   between the three runs): its id and how are in `tests/wrong.tsv`;
//! - it is refused (the pipeline says it does not implement something): in neither.
//!
//! The test fails when a case is not where the lists say, so the lists are always the
//! truth: `passing.txt` only grows and `wrong.tsv` only shrinks, by fixing. What it found
//! is written to `target/cpl-next/`.

#[path = "../../cpl-cli/tests/support/cxx.rs"]
mod cxx;
#[path = "../../cpl-cli/tests/support/parallel.rs"]
mod parallel;

use cpl_conform::conformance::{
    discover, header_input, header_values, parse_header, unquote, Expect,
};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(10);
const CXX_FLAGS: [&str; 8] = [
    "-std=c++17",
    "-O1",
    "-g",
    "-fsanitize=address,undefined",
    "-fno-sanitize-recover=undefined",
    "-w",
    "-pthread",
    "-x",
];

struct Ran {
    /// `None`: killed at the limit.
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(mut cmd: Command, input: &[u8]) -> Ran {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the program starts");
    // The input may be gone before a program that fails early reads it.
    let _ = child.stdin.take().unwrap().write_all(input);
    let (out, err) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());
    let read = |mut pipe: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut s = Vec::new();
            let _ = pipe.read_to_end(&mut s);
            String::from_utf8_lossy(&s).into_owned()
        })
    };
    let (out, err) = (read(Box::new(out)), read(Box::new(err)));
    let mut exit = cpl_wait::on_exit(child);
    let status = match exit.within(LIMIT).expect("waiting works") {
        Some(s) => Some(s.code().unwrap_or(-1)),
        None => {
            let _ = exit.kill();
            None
        }
    };
    Ran {
        status,
        stdout: out.join().unwrap(),
        stderr: err.join().unwrap(),
    }
}

fn next(path: &Path, flags: &[&str], input: &[u8]) -> Ran {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cpl-next"));
    cmd.arg(path).args(flags);
    run(cmd, input)
}

/// Header keys that ask for something this harness does not do yet (a second file, a judge
/// profile, a command run before the tool, a copy count, text the output must or must not
/// hold, more than the 10 seconds every run gets): a case with one counts as refused.
/// Every other `// word:` line is a key the harness acts on (`expect`, `input`, `tool`,
/// `output`) or a comment, so a note added to a header never changes what a case is.
const ASKS: [&str; 7] = [
    "file", "judge", "before", "copies", "has", "lacks", "timeout",
];

/// What a case asks cpl-next for: a run, or only the checks.
#[derive(Clone, Copy, PartialEq)]
enum Ask {
    Run,
    Check,
}

/// `// tool: run` and `check`, each on the case, are the pipeline's own queries; every other
/// tool or flag needs a facade query cpl-next has not got (`build` is `emit_cpp(profile)`,
/// whose judge banner and size check are not the bare C++ `--cpp` prints).
fn ask(tool: Option<&str>) -> Result<Ask, String> {
    let Some(args) = tool else {
        return Ok(Ask::Run);
    };
    let words: Vec<&str> = args.split_whitespace().filter(|w| *w != "{case}").collect();
    match words.as_slice() {
        ["run"] => Ok(Ask::Run),
        ["check"] => Ok(Ask::Check),
        ["lint", ..] => Err("`cpl lint`: no lint query yet".into()),
        _ => Err(format!("`cpl {args}`: no facade query for it yet")),
    }
}

/// `// key: ...` gives `key`.
fn header_key(line: &str) -> Option<&str> {
    let (key, _) = line.strip_prefix("//")?.trim_start().split_once(':')?;
    key.bytes()
        .all(|b| b.is_ascii_lowercase() || b == b'-')
        .then_some(key)
}

/// `stage: program:N:C: E0123: text` gives `(N, "E0123")`: the line of the span's start in
/// the case's file, and the code.
fn line_and_code(diag: &str) -> Option<(usize, &str)> {
    let rest = diag.split_once(": program:")?.1;
    let (line, rest) = rest.split_once(':')?;
    let rest = rest.split_once(": ")?.1;
    Some((line.parse().ok()?, rest.split(':').next()?))
}

/// `program:3:5:` gives `program`.
fn place_file(word: &str) -> Option<&str> {
    let (file, pos) = word.strip_suffix(':')?.split_once(':')?;
    (pos.split(':').count() == 2 && pos.split(':').all(|n| n.parse::<u32>().is_ok()))
        .then_some(file)
}

/// How one case fared: `pass`, `refused`, or the word `wrong.tsv` uses, and the detail.
/// A case that passes on its `// expect:` passes only if what the deciding run printed is
/// its `// output:`, when it gives one.
fn verdict(path: &Path) -> (&'static str, String) {
    let text = std::fs::read_to_string(path).unwrap();
    let expect = match parse_header(&text) {
        Ok(e) => e,
        Err(e) => return ("bad-header", e),
    };
    let header = text.lines().take_while(|l| l.starts_with("//"));
    if let Some(key) = header.filter_map(header_key).find(|k| ASKS.contains(k)) {
        return ("refused", format!("a `// {key}:` case"));
    }
    let ask = match ask(header_values(&text, "tool:").first().copied()) {
        Ok(a) => a,
        Err(e) => return ("refused", e),
    };
    if let Expect::Exit(n @ (2 | 4..)) = expect {
        return (
            "refused",
            format!("exit {n}: the command line's own status"),
        );
    }
    if matches!(&expect, Expect::Diag { code, .. } if code.starts_with('L')) {
        return ("refused", "a lint".into());
    }
    let (verdict, detail, printed) = judged(path, &text, expect, ask);
    if verdict != "pass" {
        return (verdict, detail);
    }
    if let Some(want) = header_values(&text, "output:").first() {
        let want = unquote(want);
        if printed != want {
            return ("wrong-output", format!("wanted {want:?}, got {printed:?}"));
        }
    }
    ("pass", String::new())
}

/// The verdict on the `// expect:` line alone, and what the run that decided it printed.
fn judged(path: &Path, text: &str, expect: Expect, ask: Ask) -> (&'static str, String, String) {
    let input = header_input(text);
    let flags: &[&str] = match ask {
        Ask::Run => &["--raw", "--trace"],
        Ask::Check => &["--check"],
    };
    let raw = next(path, flags, &input);
    let first = |s: &str| s.lines().next().unwrap_or("").to_string();
    let done = |v: &'static str, d: String| (v, d, String::new());
    match raw.status {
        None => return done("timeout", "the interpreter passed 10 s".into()),
        Some(4) => return done("refused", first(&raw.stdout)),
        Some(0 | 1 | 3) => {}
        Some(n) => {
            return done(
                "crash",
                format!("the interpreter exited {n}: {}", first(&raw.stderr)),
            )
        }
    }
    let printed = format!("{}{}", raw.stdout, raw.stderr);
    let pass = || ("pass", String::new(), printed.clone());
    if raw.status == Some(1) {
        return match &expect {
            Expect::Diag { code, line }
                if raw
                    .stdout
                    .lines()
                    .any(|d| line_and_code(d) == Some((*line, code.as_str()))) =>
            {
                pass()
            }
            Expect::Exit(1) => pass(),
            Expect::Diag { code, line } => done(
                "wrong-error",
                format!("wanted {code} on line {line}, got: {}", first(&raw.stdout)),
            ),
            _ => done("wrong-reject", first(&raw.stdout)),
        };
    }
    match &expect {
        Expect::Diag { code, line } => {
            return done(
                "wrong-accept",
                format!(
                    "wanted {code} on line {line}, ran and printed: {}",
                    first(&raw.stdout)
                ),
            )
        }
        Expect::Exit(1) => return done("wrong-accept", "wanted exit 1".into()),
        Expect::Trap { .. } | Expect::Exit(3) if ask != Ask::Run => {
            return done(
                "wrong-output",
                "a trap asked of a command that runs nothing".into(),
            )
        }
        Expect::Ok | Expect::Exit(0) if ask != Ask::Run => return pass(),
        _ => {}
    }
    let canon = next(path, &["--trace"], &input);
    let same =
        |a: &Ran, b: &Ran| a.status == b.status && a.stdout == b.stdout && a.stderr == b.stderr;
    if !same(&raw, &canon) {
        return done(
            "split",
            format!(
                "canonical IR: exit {:?}: {}",
                canon.status,
                first(&canon.stdout)
            ),
        );
    }
    let cpp = next(path, &["--cpp", "--trace"], b"");
    let bin = match cxx::build(cpp.stdout.as_bytes(), &[&CXX_FLAGS[..], &["c++"]].concat()) {
        Ok((bin, _)) => bin,
        Err(e) => return done("no-cxx", e.lines().take(4).collect::<Vec<_>>().join(" | ")),
    };
    let built = run(Command::new(bin), &input);
    if !same(&raw, &built) {
        return done(
            "split",
            format!(
                "C++: exit {:?}: {} {}",
                built.status,
                first(&built.stdout),
                first(&built.stderr)
            ),
        );
    }
    let trapped = raw.status == Some(3);
    match expect {
        Expect::Ok | Expect::Exit(0) if !trapped => pass(),
        Expect::OkOutput(want) if !trapped && raw.stdout.trim_end_matches('\n') == want => pass(),
        Expect::Exit(3) if trapped => pass(),
        Expect::Ok | Expect::Exit(0) => {
            done("wrong-trap", first(raw.stderr.lines().last().unwrap_or("")))
        }
        Expect::OkOutput(_) if trapped => {
            done("wrong-trap", first(raw.stderr.lines().last().unwrap_or("")))
        }
        Expect::OkOutput(want) => done(
            "wrong-output",
            format!("wanted {want:?}, got {:?}", raw.stdout),
        ),
        Expect::Trap { code, line } if trapped => {
            let at = format!("trap: {code}: line {line}:");
            if raw.stderr.lines().any(|l| l.starts_with(&at)) {
                pass()
            } else {
                done(
                    "wrong-trap",
                    format!(
                        "wanted {code} on line {line}, got: {}",
                        raw.stderr.lines().last().unwrap_or("")
                    ),
                )
            }
        }
        Expect::Trap { code, line } => done(
            "wrong-output",
            format!("wanted {code} on line {line}, ran to the end"),
        ),
        Expect::Exit(3) => done("wrong-output", "wanted a trap, ran to the end".into()),
        Expect::Diag { .. } | Expect::Exit(_) => {
            done("wrong-output", "an expectation settled above".into())
        }
    }
}

fn list(path: &Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
        .map(|l| match l.split_once('\t') {
            Some((id, how)) => (
                id.to_string(),
                how.split('\t').next().unwrap_or(how).to_string(),
            ),
            None => (l.to_string(), String::new()),
        })
        .collect()
}

#[test]
fn every_case_is_where_the_lists_say() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = here.parent().and_then(Path::parent).unwrap();
    let mut cases: Vec<(String, PathBuf)> = discover(root, None)
        .into_iter()
        .map(|c| (c.id, c.path))
        .collect();
    let own = here.join("tests/programs");
    for e in std::fs::read_dir(&own).unwrap() {
        let path = e.unwrap().path();
        if path.extension().is_some_and(|x| x == "cp") {
            cases.push((
                format!("next/{}", path.file_name().unwrap().to_str().unwrap()),
                path,
            ));
        }
    }
    cases.sort();
    let only = cpl_env::NEXT_ONLY.text();
    if let Some(only) = &only {
        cases.retain(|(id, _)| only.split(',').any(|part| id.contains(part)));
    } else {
        assert!(
            cases.len() > 4000,
            "only {} cases: the conformance suite was not found",
            cases.len()
        );
    }
    let verdicts = parallel::parallel(&cases, |(_, p)| verdict(p));

    let (mut passing, mut wrong, mut refused) = (String::new(), String::new(), String::new());
    for ((id, _), (verdict, detail)) in cases.iter().zip(&verdicts) {
        match *verdict {
            "pass" => passing += &format!("{id}\n"),
            "refused" => refused += &format!("{id}\t{detail}\n"),
            how => wrong += &format!("{id}\t{how}\t{detail}\n"),
        }
    }
    let found = root.join(if only.is_some() {
        "target/cpl-next/only"
    } else {
        "target/cpl-next"
    });
    std::fs::create_dir_all(&found).unwrap();
    let queue = queue(&refused);
    for (name, text) in [
        ("passing.txt", &passing),
        ("wrong.tsv", &wrong),
        ("refused.tsv", &refused),
        ("queue.txt", &queue),
    ] {
        std::fs::write(found.join(name), text).unwrap();
    }

    let (listed_passing, listed_wrong) = (
        list(&here.join("tests/passing.txt")),
        list(&here.join("tests/wrong.tsv")),
    );
    let mut off = Vec::new();
    let mut seen = Vec::new();
    for ((id, _), (verdict, detail)) in cases.iter().zip(&verdicts) {
        seen.push(id.as_str());
        let listed = if listed_passing.contains_key(id) {
            Some("pass")
        } else {
            listed_wrong.get(id).map(String::as_str)
        };
        match (listed, *verdict) {
            (Some(l), got) if l == got => {}
            (None, "refused") => {}
            (None, "pass") => off.push(format!("{id}: passes: add it to tests/passing.txt")),
            (Some("pass"), got) => off.push(format!("{id}: passed, now {got}: {detail}")),
            (Some(l), "pass") => off.push(format!(
                "{id}: passes: move it from tests/wrong.tsv ({l}) to tests/passing.txt"
            )),
            (Some(l), "refused") => off.push(format!(
                "{id}: refused now: take its `{l}` line out of tests/wrong.tsv"
            )),
            (Some(l), got) => off.push(format!("{id}: listed as {l}, now {got}: {detail}")),
            (None, got) => off.push(format!("{id}: {got}: {detail}")),
        }
    }
    for id in listed_passing
        .keys()
        .chain(listed_wrong.keys())
        .filter(|id| only.is_none() && !seen.contains(&id.as_str()))
    {
        off.push(format!("{id}: listed, but there is no such case"));
    }
    let count = |s: &str| s.lines().count();
    println!(
        "cpl-next: {} cases: {} pass, {} wrong, {} refused",
        cases.len(),
        count(&passing),
        count(&wrong),
        count(&refused)
    );
    println!("refused, by cause (all in target/cpl-next/queue.txt):");
    queue.lines().take(10).for_each(|l| println!("  {l}"));
    assert!(
        off.is_empty(),
        "{} of {} cases are not where the lists say (what was found is in {}):\n{}",
        off.len(),
        cases.len(),
        found.display(),
        off.join("\n")
    );
}

/// The refused cases grouped by what refused them, largest group first: the order the
/// missing mechanisms are built in. A cause is its message with places in the program and
/// quoted names taken out, so `unknown name `V`` and `unknown name `W`` are one cause; a
/// place in a library keeps its module.
fn queue(refused: &str) -> String {
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
    for line in refused.lines() {
        let detail = line.split_once('\t').map_or("", |(_, d)| d);
        let mut cause = String::new();
        let mut quoted = false;
        for part in detail.split('`') {
            cause += if quoted { "`_`" } else { part };
            quoted = !quoted;
        }
        let cause: Vec<&str> = cause
            .split_whitespace()
            .filter_map(|w| match place_file(w) {
                Some("program") => None,
                Some(file) => Some(file),
                None => Some(w),
            })
            .filter(|w| !w.chars().all(|c| c.is_ascii_digit() || c == ':'))
            .collect();
        *by.entry(cause.join(" ")).or_default() += 1;
    }
    let mut by: Vec<(String, usize)> = by.into_iter().collect();
    by.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    by.iter().map(|(c, n)| format!("{n:5} {c}\n")).collect()
}
