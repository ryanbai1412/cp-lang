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

fn run(cmd: Command, input: &[u8]) -> Ran {
    run_within(cmd, input, LIMIT)
}

fn run_within(mut cmd: Command, input: &[u8], limit: Duration) -> Ran {
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
    let status = match exit.within(limit).expect("waiting works") {
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

/// A case's runs from a `cpl-next --all` worker, which reads std once and compiles each case
/// it is given against it, then runs every form in turn, each within what [`LIMIT`] leaves
/// after the compile, as a process of its own would. A run it did not finish (a crash, the
/// limit) ends the worker and is run again alone, so sharing never changes what a run gives.
struct Forms<'a> {
    dir: PathBuf,
    path: &'a Path,
    input: &'a [u8],
    /// What the compile printed when it rejected or refused the case.
    compiled: Ran,
}

/// A `cpl-next --all` process and its `done` lines.
struct Worker {
    child: std::process::Child,
    ask: std::process::ChildStdin,
    done: std::sync::mpsc::Receiver<String>,
}

impl Worker {
    fn new() -> Worker {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cpl-next"))
            .args([
                "--trace",
                "--all",
                "--within",
                &LIMIT.as_millis().to_string(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("cpl-next starts");
        let ask = child.stdin.take().unwrap();
        let out = std::io::BufReader::new(child.stdout.take().unwrap());
        let (said, done) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufRead::lines(out).map_while(Result::ok) {
                if said.send(line).is_err() {
                    break;
                }
            }
        });
        Worker { child, ask, done }
    }

    /// Whether it finished the case: its three runs stop themselves, a compile is given the
    /// rest of the four limits a process of its own had.
    fn finished(&mut self, path: &Path, dir: &Path) -> bool {
        writeln!(self.ask, "{}\t{}", path.display(), dir.display()).is_ok()
            && self.done.recv_timeout(4 * LIMIT).is_ok_and(|l| l == "done")
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

thread_local! {
    static WORKER: std::cell::RefCell<Option<Worker>> = const { std::cell::RefCell::new(None) };
}

impl<'a> Forms<'a> {
    fn new(path: &'a Path, input: &'a [u8]) -> Self {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("forms")
            .join(format!("{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the forms directory is made");
        std::fs::write(dir.join("in"), input).expect("the input is written");
        WORKER.with_borrow_mut(|w| {
            let finished = w.get_or_insert_with(Worker::new).finished(path, &dir);
            if !finished {
                *w = None;
            }
        });
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
        let compiled = Ran {
            status: read("compiled.code").and_then(|c| c.parse().ok()),
            stdout: read("compiled.out").unwrap_or_default(),
            stderr: String::new(),
        };
        Forms {
            dir,
            path,
            input,
            compiled,
        }
    }

    fn read(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.dir.join(name)).ok()
    }

    fn form(&self, name: &str, flags: &[&str]) -> Ran {
        let kept = |x: &str| self.read(&format!("{name}.{x}"));
        match (kept("code"), kept("out"), kept("err")) {
            (Some(code), Some(stdout), Some(stderr)) => Ran {
                status: code.parse().ok(),
                stdout,
                stderr,
            },
            _ => next(self.path, flags, self.input),
        }
    }

    /// The raw run, or the compile's diagnostics when it rejected or refused the case.
    fn raw(&self) -> Ran {
        match self.compiled.status {
            Some(1 | 4) if self.read("raw.code").is_none() => Ran {
                status: self.compiled.status,
                stdout: self.compiled.stdout.clone(),
                stderr: self.compiled.stderr.clone(),
            },
            _ => self.form("raw", &["--raw", "--trace"]),
        }
    }

    fn cpp(&self) -> String {
        match self.read("c++") {
            Some(cpp) => cpp,
            None => next(self.path, &["--cpp", "--trace"], b"").stdout,
        }
    }
}

impl Drop for Forms<'_> {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// FNV-1a, as the build cache keys by.
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h: u64, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// `bin` run on `input`, or the same run kept from before: the same binary bytes on the same
/// input give the same run. Kept under `target/cpl-next/runs` by a hash of both, with the
/// input itself, compared before a hit; written under a name of its own and renamed, so
/// concurrent suites never read half a run. A run killed at the limit is not kept.
fn run_kept(bin: &Path, input: &[u8]) -> Ran {
    let runs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/cpl-next/runs");
    let binary = std::fs::read(bin).expect("the built binary is readable");
    let kept = runs.join(format!("{:016x}-{:016x}", fnv(&binary), fnv(input)));
    if let Some(ran) = std::fs::read(&kept).ok().and_then(|b| unkeep(&b, input)) {
        return ran;
    }
    let ran = run(Command::new(bin), input);
    if let Some(code) = ran.status {
        let mut b = format!("{code} {} {}\n", input.len(), ran.stdout.len()).into_bytes();
        b.extend_from_slice(input);
        b.extend_from_slice(ran.stdout.as_bytes());
        b.extend_from_slice(ran.stderr.as_bytes());
        let tmp = runs.join(format!(
            "{}.{}.{:?}",
            kept.file_name().unwrap().to_str().unwrap(),
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::create_dir_all(&runs)
            .and_then(|()| std::fs::write(&tmp, &b))
            .and_then(|()| std::fs::rename(&tmp, &kept));
    }
    ran
}

/// A kept run, if it was kept for this input.
fn unkeep(b: &[u8], input: &[u8]) -> Option<Ran> {
    let end = b.iter().position(|&c| c == b'\n')?;
    let head = std::str::from_utf8(&b[..end]).ok()?;
    let mut nums = head.split(' ');
    let code = nums.next()?.parse().ok()?;
    let (inl, outl): (usize, usize) = (nums.next()?.parse().ok()?, nums.next()?.parse().ok()?);
    let rest = &b[end + 1..];
    if rest.get(..inl)? != input {
        return None;
    }
    let out = rest.get(inl..inl + outl)?;
    let err = &rest[inl + outl..];
    Some(Ran {
        status: Some(code),
        stdout: String::from_utf8(out.to_vec()).ok()?,
        stderr: String::from_utf8(err.to_vec()).ok()?,
    })
}

/// Header keys that ask for something this harness does not do yet (a judge profile, a command run before the tool, a copy count, text the output must or must not
/// hold, more than the 10 seconds every run gets): a case with one counts as refused.
/// Every other `// word:` line is a key the harness acts on (`expect`, `input`, `tool`,
/// `output`) or a comment, so a note added to a header never changes what a case is.
const ASKS: [&str; 4] = ["judge", "before", "copies", "timeout"];

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

/// `E0302` and the like: a code, not a refusal's `slice`.
fn coded(word: &str) -> bool {
    word.len() == 5 && word.starts_with('E') && word[1..].bytes().all(|b| b.is_ascii_digit())
}

/// `program:3:5:` gives `program`.
fn place_file(word: &str) -> Option<&str> {
    let (file, pos) = word.strip_suffix(':')?.split_once(':')?;
    (pos.split(':').count() == 2 && pos.split(':').all(|n| n.parse::<u32>().is_ok()))
        .then_some(file)
}

/// How one case fared: `pass`, `refused`, `masked` (refused beside a coded verdict), or the word
/// `wrong.tsv` uses, and the detail.
/// A case that passes on its `// expect:` passes only if what the deciding run printed is
/// its `// output:`, when it gives one.
fn verdict(path: &Path) -> (&'static str, String) {
    let text = std::fs::read_to_string(path).unwrap();
    let expect = match parse_header(&text) {
        Ok(e) => e,
        Err(e) => return ("bad-header", e),
    };
    let header = || text.lines().take_while(|l| l.starts_with("//"));
    if let Some(key) = header().filter_map(header_key).find(|k| ASKS.contains(k)) {
        return ("refused", format!("a `// {key}:` case"));
    }
    // Text a case asks to see is judged once the case otherwise passes; until then the
    // case stays refused, as the text may come from a report the pipeline lacks.
    let asks_text = header()
        .filter_map(header_key)
        .find(|k| ["has", "lacks"].contains(k));
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
    let scratch = match beside_its_files(path, &text) {
        Ok(s) => s,
        Err(e) => return ("bad-header", e),
    };
    let run_at = scratch
        .as_ref()
        .map_or(path.to_path_buf(), |d| d.join(path.file_name().unwrap()));
    let (verdict, detail, printed) = judged(&run_at, &text, expect, ask);
    if let Some(d) = scratch {
        let _ = std::fs::remove_dir_all(d);
    }
    if let Some(key) = asks_text {
        let refused = || ("refused", format!("a `// {key}:` case"));
        if verdict != "pass" {
            return refused();
        }
        let mut has = header_values(&text, "has:").into_iter().map(unquote);
        let mut lacks = header_values(&text, "lacks:").into_iter().map(unquote);
        if !has.all(|w| printed.contains(&w)) || lacks.any(|n| printed.contains(&n)) {
            return refused();
        }
    }
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

/// A case with `// file: PATH "TEXT"` lines runs from a scratch folder holding a copy of it
/// and those files, which its `cp.toml` and library modules are read from.
fn beside_its_files(path: &Path, text: &str) -> Result<Option<PathBuf>, String> {
    let files = header_values(text, "file:");
    if files.is_empty() {
        return Ok(None);
    }
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let name: String = (path.components().rev().take(2))
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("-");
    let dir = here.join("../../target/cpl-next/scratch").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(path.file_name().unwrap()), text).map_err(|e| e.to_string())?;
    for spec in files {
        let (rel, body) = (spec.split_once(char::is_whitespace))
            .ok_or("`// file:` needs a path and a quoted body")?;
        if rel.starts_with('/') || rel.split('/').any(|p| p == "..") {
            return Err(format!(
                "`// file: {rel}` must stay inside the scratch folder"
            ));
        }
        let at = dir.join(rel);
        std::fs::create_dir_all(at.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&at, unquote(body.trim())).map_err(|e| e.to_string())?;
    }
    Ok(Some(dir))
}

/// The verdict on the `// expect:` line alone, and what the run that decided it printed.
fn judged(path: &Path, text: &str, expect: Expect, ask: Ask) -> (&'static str, String, String) {
    let input = header_input(text);
    let forms = (ask == Ask::Run).then(|| Forms::new(path, &input));
    let raw = match &forms {
        Some(forms) => forms.raw(),
        None => next(path, &["--check"], &input),
    };
    let first = |s: &str| s.lines().next().unwrap_or("").to_string();
    let done = |v: &'static str, d: String| (v, d, String::new());
    match raw.status {
        None => return done("timeout", "the interpreter passed 10 s".into()),
        // A refusal beside a coded verdict on the program hides that verdict: the case is
        // `masked`, refused for the lists but counted apart (tests/masked.txt).
        Some(4)
            if raw
                .stdout
                .lines()
                .any(|d| line_and_code(d).is_some_and(|(_, c)| coded(c))) =>
        {
            return done("masked", first(&raw.stdout))
        }
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
    let forms = forms.unwrap_or_else(|| Forms::new(path, &input));
    let canon = forms.form("canonical", &["--trace"]);
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
    let engine = forms.form("engine", &["--engine", "--trace"]);
    if !same(&canon, &engine) {
        return done(
            "split",
            format!(
                "engine: exit {:?}: {} {}",
                engine.status,
                first(&engine.stdout),
                first(&engine.stderr)
            ),
        );
    }
    let cpp = forms.cpp();
    let bin = match cxx::build(cpp.as_bytes(), &[&CXX_FLAGS[..], &["c++"]].concat()) {
        Ok((bin, _)) => bin,
        Err(e) => return done("no-cxx", e.lines().take(4).collect::<Vec<_>>().join(" | ")),
    };
    let built = run_kept(&bin, &input);
    // A trap's call stack comes from the engines' events; generated C++ keeps no frames.
    fn judged(r: &Ran) -> (Option<i32>, &str, Vec<&str>) {
        let lines = r.stderr.lines().filter(|l| !l.starts_with("call stack: "));
        (r.status, &r.stdout, lines.collect())
    }
    if judged(&raw) != judged(&built) {
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
        Expect::Ok | Expect::Exit(0) => done("wrong-trap", first(trap_line(&raw.stderr))),
        Expect::OkOutput(_) if trapped => done("wrong-trap", first(trap_line(&raw.stderr))),
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
                        trap_line(&raw.stderr)
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

/// A kept build or run is never returned for other bytes: changed C++ is built again, a
/// kept C++ that differs from the one asked for is rebuilt, and a binary that changed under
/// the same name is run again.
#[test]
fn a_kept_build_or_run_is_never_stale() {
    let flags = ["-x", "c++"];
    let say = |word: &str| {
        format!(
            "#include <cstdio>\nint main() {{ std::puts(\"{word}\"); }}\n// {}\n",
            std::process::id()
        )
    };
    let (one, two) = (say("one"), say("two"));
    let built = |cpp: &str| cxx::build(cpp.as_bytes(), &flags).expect("g++ builds it").0;
    let printed = |bin: &Path| run_kept(bin, b"").stdout;
    let first = built(&one);
    assert_eq!(printed(&first), "one\n");
    let second = built(&two);
    assert_ne!(first, second);
    assert_eq!(printed(&second), "two\n");

    std::fs::copy(&second, &first).unwrap();
    assert_eq!(
        printed(&first),
        "two\n",
        "a run kept for other binary bytes"
    );

    std::fs::write(first.with_extension("cpp"), &two).unwrap();
    assert_eq!(built(&one), first);
    assert_eq!(printed(&first), "one\n", "a build kept for other C++");
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
    let mut masked = String::new();
    for ((id, _), (verdict, detail)) in cases.iter().zip(&verdicts) {
        match *verdict {
            "pass" => passing += &format!("{id}\n"),
            "masked" => {
                masked += &format!("{id}\n");
                refused += &format!("{id}\t{detail}\n");
            }
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
        ("masked.txt", &masked),
        ("queue.txt", &queue),
    ] {
        std::fs::write(found.join(name), text).unwrap();
    }

    let (listed_passing, listed_wrong, listed_masked) = (
        list(&here.join("tests/passing.txt")),
        list(&here.join("tests/wrong.tsv")),
        list(&here.join("tests/masked.txt")),
    );
    let mut off = Vec::new();
    let mut seen = Vec::new();
    let mut freed = 0;
    for ((id, _), (verdict, detail)) in cases.iter().zip(&verdicts) {
        seen.push(id.as_str());
        match (listed_masked.contains_key(id), *verdict == "masked") {
            (false, true) => off.push(format!(
                "{id}: refused beside a coded verdict on the program ({detail}): add it to tests/masked.txt"
            )),
            // A case leaving the list only lowers what it counts, so it need not stop a landing
            // that raced the change which freed it; target/cpl-next/masked.txt is the exact list.
            (true, false) => freed += 1,
            _ => {}
        }
        let verdict = &if *verdict == "masked" {
            "refused"
        } else {
            *verdict
        };
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
        .chain(listed_masked.keys())
        .filter(|id| only.is_none() && !seen.contains(&id.as_str()))
    {
        off.push(format!("{id}: listed, but there is no such case"));
    }
    let count = |s: &str| s.lines().count();
    println!(
        "cpl-next: {} cases: {} pass, {} wrong, {} refused ({} of them beside a coded verdict)",
        cases.len(),
        count(&passing),
        count(&wrong),
        count(&refused),
        count(&masked)
    );
    if freed > 0 {
        println!("{freed} cases in tests/masked.txt no longer hide a verdict: copy target/cpl-next/masked.txt over it");
    }
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

/// The line saying how a run trapped: `trap: CODE: line N: message`.
fn trap_line(stderr: &str) -> &str {
    stderr
        .lines()
        .find(|l| l.starts_with("trap: "))
        .unwrap_or("")
}
