//! Every conformance case under `spec/conformance`, and every program in `tests/programs`
//! (the same header format), against the result its header states. The header comes from
//! the spec, not from what this crate or today's compiler prints.
//!
//! A case that runs is run three ways on its `// input:`: the reference interpreter on the
//! linked instances, the same on canonical IR, and the emitted C++ under ASan and UBSan.
//! All three give the same stdout, exit status and events, within 10 seconds each, or the
//! longer `// timeout:` a case's header gives.
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
use cpl_next::facade::{Session, Snapshot, Verdict, World};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

// The allocator cpl-next ships with: the in-process checks run on it as the binaries do.
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

/// The judge a case is built for, as `cpl-next --judge` takes it: the harness resolves it the
/// way a build does, from the `cp.toml` a `// file:` header put beside the case, or else as
/// one with no project config; the compiler only reads it.
fn judge(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let p = match path.parent().is_some_and(|d| d.join("cp.toml").is_file()) {
        true => cpl_judge::resolve(path, &text, None).ok()?,
        false => cpl_judge::of(&text)?,
    };
    Some(format!("{}={}", p.name, p.memory_mb))
}

/// `cpl-next`, or for a reference interpreter's run (`--raw`, `--canonical`) the tests'
/// `cpl-next-oracle`.
fn next(path: &Path, flags: &[&str], input: &[u8], limit: Duration) -> Ran {
    let bin = match flags.iter().any(|f| matches!(*f, "--raw" | "--canonical")) {
        true => env!("CARGO_BIN_EXE_cpl-next-oracle"),
        false => env!("CARGO_BIN_EXE_cpl-next"),
    };
    let mut cmd = Command::new(bin);
    cmd.arg(path).args(flags);
    if let Some(j) = judge(path) {
        cmd.args(["--judge", &j]);
    }
    run_within(cmd, input, limit)
}

/// A case's runs from a `cpl-next-oracle --all` worker, which reads std once and compiles each case
/// it is given against it, then runs every form in turn, each within what [`LIMIT`] leaves
/// after the compile, as a process of its own would. A run it did not finish (a crash, the
/// limit) ends the worker and is run again alone, so sharing never changes what a run gives.
/// A case given a longer limit than the worker's runs alone.
struct Forms<'a> {
    dir: PathBuf,
    path: &'a Path,
    input: &'a [u8],
    limit: Duration,
    /// What the compile printed when it rejected or refused the case.
    compiled: Ran,
}

/// A `cpl-next-oracle --all` process and its `done` lines.
struct Worker {
    child: std::process::Child,
    ask: std::process::ChildStdin,
    done: std::sync::mpsc::Receiver<String>,
}

impl Worker {
    fn new() -> Worker {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cpl-next-oracle"))
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
            .expect("cpl-next-oracle starts");
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
        let judge = judge(path).unwrap_or_default();
        writeln!(self.ask, "{}\t{}\t{judge}", path.display(), dir.display()).is_ok()
            && self.done.recv_timeout(4 * LIMIT).is_ok_and(|l| l == "done")
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The cases, by a hash of their text, whose debug compile linked an instance holding an
/// error type (`Compiled::error_typed`).
static ERROR_TYPED: std::sync::Mutex<std::collections::BTreeSet<u64>> =
    std::sync::Mutex::new(std::collections::BTreeSet::new());

thread_local! {
    static WORKER: std::cell::RefCell<Option<Worker>> = const { std::cell::RefCell::new(None) };
}

impl<'a> Forms<'a> {
    fn new(path: &'a Path, input: &'a [u8], limit: Duration) -> Self {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("forms")
            .join(format!("{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the forms directory is made");
        std::fs::write(dir.join("in"), input).expect("the input is written");
        if limit == LIMIT {
            WORKER.with_borrow_mut(|w| {
                let finished = w.get_or_insert_with(Worker::new).finished(path, &dir);
                if !finished {
                    *w = None;
                }
            });
        }
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
        if read("error-typed").is_some_and(|t| !t.is_empty()) {
            let text = std::fs::read(path).unwrap_or_default();
            ERROR_TYPED.lock().unwrap().insert(fnv(&text));
        }
        let compiled = Ran {
            status: read("compiled.code").and_then(|c| c.parse().ok()),
            stdout: read("compiled.out").unwrap_or_default(),
            stderr: String::new(),
        };
        Forms {
            dir,
            path,
            input,
            limit,
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
            _ => next(self.path, flags, self.input, self.limit),
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
            _ => self.form("raw", &["--raw", "--trace", "--json"]),
        }
    }

    fn cpp(&self) -> String {
        match self.read("c++") {
            Some(cpp) => cpp,
            None => next(self.path, &["--cpp", "--trace"], b"", self.limit).stdout,
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
fn run_kept(bin: &Path, input: &[u8], limit: Duration) -> Ran {
    run_kept_on(bin, None, input, limit)
}

/// [`run_kept`] on the judge's stack (`ulimit -s`), as a release run gets it (judge.8).
fn run_kept_on(
    bin: &Path,
    judge: Option<&cpl_judge::Profile>,
    input: &[u8],
    limit: Duration,
) -> Ran {
    let runs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/cpl-next/runs");
    let binary = std::fs::read(bin).expect("the built binary is readable");
    let stack = judge.map(|j| j.ulimit()).unwrap_or_default();
    let kept = runs.join(format!(
        "{:016x}-{:016x}{}",
        fnv(&binary),
        fnv(input),
        if stack.is_empty() {
            String::new()
        } else {
            format!("-s{stack}")
        }
    ));
    if let Some(ran) = std::fs::read(&kept).ok().and_then(|b| unkeep(&b, input)) {
        return ran;
    }
    let cmd = match judge {
        Some(j) => {
            let words = j.on_stack(bin.display().to_string());
            let mut cmd = Command::new(&words[0]);
            cmd.args(&words[1..]);
            cmd
        }
        None => Command::new(bin),
    };
    let ran = run_within(cmd, input, limit);
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

/// The limit each of a case's runs gets: [`LIMIT`], or its `// timeout: <secs>` when that is
/// longer (a trap case that has to exhaust a budget first).
fn limit_of(text: &str) -> Duration {
    (header_values(text, "timeout:").iter())
        .filter_map(|v| v.parse().ok())
        .map(Duration::from_secs)
        .fold(LIMIT, Duration::max)
}

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
        // A release run is the C++ build, which every run is compared with.
        ["run"] | ["run", "--release"] => Ok(Ask::Run),
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

/// A verdict on the case's own file, from `cpl-next --json`: the line its span starts on and
/// its code. A refusal, or a place in a library, is `None`.
fn on_the_program(diag: &cpl_json::Value) -> Option<(usize, &str)> {
    let field = |k: &str| diag.get(k);
    (field("file")?.as_str()? == "program" && field("refusal")?.is_null()).then_some(())?;
    let line = field("start")?.get("line")?.as_u64()? as usize;
    Some((line, field("code")?.as_str()?))
}

/// A rejection's `--json` lines, and `ran` with what they say in place of them.
fn diagnosed(ran: Ran) -> (Ran, Vec<cpl_json::Value>) {
    if !matches!(ran.status, Some(1 | 4)) {
        return (ran, Vec::new());
    }
    let diags: Vec<_> = (ran.stdout.lines())
        .map(|l| cpl_json::parse(l).unwrap_or_else(|e| panic!("`{l}` is not a diagnostic: {e}")))
        .collect();
    let text = |d: &cpl_json::Value| {
        d.get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string()
    };
    let stdout = diags.iter().map(|d| text(d) + "\n").collect();
    (Ran { stdout, ..ran }, diags)
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
    let limit = limit_of(&text);
    // A `// before:` tool writes files only a later tool reads, so one cpl-next has is no step.
    if let Some(e) = (header_values(&text, "before:").into_iter()).find_map(|b| ask(Some(b)).err())
    {
        return ("refused", format!("before: {e}"));
    }
    // Text a case asks to see is judged once the case otherwise passes; a case that doesn't
    // keeps its verdict, so a wrong-accept or wrong-reject stays wrong (shortcut audit S14).
    let asks_text = header()
        .filter_map(header_key)
        .find(|k| ["has", "lacks"].contains(k));
    let ask = match ask(header_values(&text, "tool:").first().copied()) {
        Ok(a) => a,
        Err(e) => return ("refused", e),
    };
    // Any rejection exits 1, so an `exit 1` case says which one it wants.
    if expect == Expect::Exit(1) && header().filter_map(header_key).all(|k| k != "has") {
        return (
            "bad-header",
            "exit 1 names no error: say which with a `// has:`".into(),
        );
    }
    if let Expect::Exit(n @ (2 | 4..)) = expect {
        return (
            "refused",
            format!("exit {n}: the command line's own status"),
        );
    }
    if matches!(&expect, Expect::Diag { code, .. } if is_lint(code)) {
        return ("refused", "a lint".into());
    }
    let scratch = match beside_its_files(path, &text) {
        Ok(s) => s,
        Err(e) => return ("bad-header", e),
    };
    let run_at = scratch
        .as_ref()
        .map_or(path.to_path_buf(), |d| d.join(path.file_name().unwrap()));
    let (verdict, detail, printed, stdout) = judged(&run_at, &text, expect, ask, limit);
    if let Some(d) = scratch {
        let _ = std::fs::remove_dir_all(d);
    }
    if asks_text.is_some() {
        if verdict != "pass" {
            return (verdict, detail);
        }
        let mut has = header_values(&text, "has:").into_iter().map(unquote);
        let mut lacks = header_values(&text, "lacks:").into_iter().map(unquote);
        let missing: Vec<String> = has.by_ref().filter(|w| !printed.contains(w)).collect();
        let extra: Vec<String> = lacks.by_ref().filter(|n| printed.contains(n)).collect();
        let said = [("missing", missing), ("printed though lacked", extra)]
            .into_iter()
            .filter(|(_, ws)| !ws.is_empty())
            .map(|(how, ws)| format!("{how} {ws:?}"))
            .collect::<Vec<_>>();
        if !said.is_empty() {
            return ("wrong-output", said.join(", "));
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
    if let Some(want) = header_values(&text, "last:").first() {
        let want = unquote(want);
        let last = stdout.lines().last().unwrap_or("");
        if last != want {
            return (
                "wrong-output",
                format!("wanted last line {want:?}, got {last:?}"),
            );
        }
    }
    // A copy count is ownership's to report, per source site, and it reports none yet: a case
    // that otherwise passes stays refused on it, and one that does not keeps its verdict.
    if !header_values(&text, "copies:").is_empty() {
        return (
            "refused",
            "a `// copies:` count: ownership reports none".into(),
        );
    }
    ("pass", String::new())
}

/// A lint's expectation: an `L` code or a named lint such as `prefix_totals_list`, which no
/// cpl-next query reports yet.
fn is_lint(code: &str) -> bool {
    code.starts_with('L') || code.starts_with(|c: char| c.is_ascii_lowercase())
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
fn judged(
    path: &Path,
    text: &str,
    expect: Expect,
    ask: Ask,
    limit: Duration,
) -> (&'static str, String, String, String) {
    let input = header_input(text);
    let forms = (ask == Ask::Run).then(|| Forms::new(path, &input, limit));
    let raw = match &forms {
        Some(forms) => forms.raw(),
        None => next(path, &["--check", "--json"], &input, limit),
    };
    let (raw, diags) = diagnosed(raw);
    let first = |s: &str| s.lines().next().unwrap_or("").to_string();
    let done = |v: &'static str, d: String| (v, d, String::new(), String::new());
    match raw.status {
        None => {
            return done(
                "timeout",
                format!("the interpreter passed {} s", limit.as_secs()),
            )
        }
        // A refusal beside a coded verdict on the program hides that verdict: the case is
        // `masked`, refused for the lists but counted apart (tests/masked.txt).
        Some(4) if diags.iter().any(|d| on_the_program(d).is_some()) => {
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
    let pass = || ("pass", String::new(), printed.clone(), raw.stdout.clone());
    if raw.status == Some(1) {
        return match &expect {
            Expect::Diag { code, line }
                if diags
                    .iter()
                    .any(|d| on_the_program(d) == Some((*line, code.as_str()))) =>
            {
                pass()
            }
            // A trap the checker finds when it works out a constant (R01.6.1).
            Expect::Trap { code, line }
                if diags
                    .iter()
                    .any(|d| on_the_program(d) == Some((*line, code.as_str()))) =>
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
    let forms = forms.unwrap_or_else(|| Forms::new(path, &input, limit));
    let canon = forms.form("canonical", &["--canonical", "--trace"]);
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
    let judge = cpl_judge::resolve(path, text, None).ok();
    let built = run_kept_on(&bin, judge.as_ref(), &input, limit);
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

/// A misplaced case's `cpl-finding` row: its id and what is wrong, without the diagnostic,
/// whose wording a land may change without moving the case.
fn finding(misplaced: &str) -> String {
    let (id, rest) = misplaced.split_once(": ").unwrap_or((misplaced, ""));
    let kind = rest.split([':', '(']).next().unwrap_or(rest).trim();
    format!("cpl-finding: every_case_is_where_the_lists_say {id} {kind} +1")
}

#[test]
fn an_exit_1_case_names_its_error() {
    let dir = std::env::temp_dir().join(format!("cpl-next-exit1-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let case = dir.join("exit1.cp");
    std::fs::write(
        &case,
        "// expect: exit 1\n// tool: check\nlet x: Int = true\n",
    )
    .unwrap();
    let got = verdict(&case);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(got.0, "bad-header", "{}", got.1);
}

#[test]
fn a_misplaced_case_is_one_row_named_by_id_and_kind() {
    for (line, row) in [
        (
            "core-01/a.cp: passed, now wrong: wanted ok, got E0302",
            "core-01/a.cp passed, now wrong",
        ),
        (
            "core-01/a.cp: refused beside a coded verdict on the program (check: E0606)",
            "core-01/a.cp refused beside a coded verdict on the program",
        ),
        (
            "m/b.cp: passes: add it to tests/passing.txt",
            "m/b.cp passes",
        ),
    ] {
        assert_eq!(
            finding(line),
            format!("cpl-finding: every_case_is_where_the_lists_say {row} +1")
        );
    }
}

/// A list's cases by id; an id listed twice (a union merge of two lands) is an error.
fn list(path: &Path) -> BTreeMap<String, String> {
    let mut cases = BTreeMap::new();
    for l in std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
    {
        let (id, how) = match l.split_once('\t') {
            Some((id, how)) => (id, how.split('\t').next().unwrap_or(how)),
            None => (l, ""),
        };
        let again = cases.insert(id.to_string(), how.to_string()).is_some();
        assert!(
            !again,
            "{}: {id} is listed twice: copy target/cpl-next/{} over it",
            path.display(),
            path.file_name().unwrap().to_string_lossy()
        );
    }
    cases
}

/// The suites whose rejections the commit tier guards: their borrow, escape and sequence
/// errors are the ones a checker change most often stops reporting (12be3511f took
/// next_wrong_accepts from 20 to 62 with no commit-tier test failing).
const GUARDED_SUITES: [&str; 3] = ["core-05/", "core-06/", "std-seq/"];

/// The commit tier's wrong-accept guard: every `// expect: E…` case of [`GUARDED_SUITES`] that
/// passing.txt lists is checked by cpl-next's checker alone, nothing run, and none may be accepted. A
/// wrong error is the full run's to judge, as is a case the lists already hold as wrong.
#[test]
fn guarded_rejections_stay_rejected() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let passing = list(&here.join("tests/passing.txt"));
    let cases: Vec<(String, PathBuf)> = all_cases()
        .into_iter()
        .filter(|(id, _)| GUARDED_SUITES.iter().any(|s| id.starts_with(s)))
        .filter(|(id, _)| passing.contains_key(id))
        .filter(|(_, path)| {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            matches!(parse_header(&text), Ok(Expect::Diag { code, .. }) if !code.starts_with('L'))
                && header_values(&text, "tool:").is_empty()
        })
        .collect();
    assert!(!cases.is_empty(), "no guarded rejection was found");
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
    let std = std::sync::Arc::new(cpl_next::Std::read());
    let snapshot = (cases.iter()).fold(
        Snapshot::new(World::Disk).with_std_lib(std),
        |s, (_, path)| {
            let text = std::fs::read_to_string(path).expect("the case is read");
            s.with_file(path.to_string_lossy(), text)
        },
    );
    let next = std::sync::atomic::AtomicUsize::new(0);
    let accepted = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            let (cases, snapshot, next, accepted) = (&cases, &snapshot, &next, &accepted);
            // A session per thread over every case: std is checked once a thread, and a case's
            // verdict reads only its own file and std.
            let ours = move || {
                let session = Session::open(snapshot.clone());
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some((id, path)) = cases.get(i) else {
                        break;
                    };
                    if session.verdict(&path.to_string_lossy()) == Verdict::Accepted {
                        accepted.lock().unwrap().push(id.clone());
                    }
                }
            };
            std::thread::Builder::new()
                .stack_size(1 << 30)
                .spawn_scoped(scope, ours)
                .expect("a checker thread starts");
        }
    });
    let mut accepted = accepted.into_inner().unwrap();
    accepted.sort();
    assert!(
        accepted.is_empty(),
        "cpl-next now accepts {} case(s) that expect an error, of {} guarded: {}",
        accepted.len(),
        cases.len(),
        accepted.join(", ")
    );
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
    let printed = |bin: &Path| run_kept(bin, b"", LIMIT).stdout;
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

/// A failed build says the same on every run and machine: it names the C++ by its key,
/// not by the temporary file, whose path holds the home directory and a thread id.
#[test]
fn a_failed_build_names_no_temporary_file() {
    let cpp = format!("int main() {{ return x; }}\n// {}\n", std::process::id());
    let said = || cxx::build(cpp.as_bytes(), &["-x", "c++"]).unwrap_err();
    let first = said();
    assert_eq!(first, said());
    let home = std::env::var("HOME").unwrap_or_default();
    assert!(!home.is_empty() && !first.contains(&home), "{first}");
}

/// Every case by id: the conformance suite's and cpl-next's own, sorted.
fn all_cases() -> Vec<(String, PathBuf)> {
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
    cases
}

/// Each case's verdict and how long it took, the cases the last full run timed slowest started
/// first, so no slow case is left running alone at the end.
fn longest_first(
    root: &Path,
    cases: &[(String, PathBuf)],
) -> Vec<(&'static str, String, Duration)> {
    let last = std::fs::read_to_string(root.join("target/cpl-next/times.tsv")).unwrap_or_default();
    let last: BTreeMap<&str, u64> = (last.lines())
        .filter_map(|l| l.split_once('\t'))
        .filter_map(|(id, ms)| Some((id, ms.trim().parse().ok()?)))
        .collect();
    let took = |id: &str| last.get(id).copied().unwrap_or(0);
    let mut order: Vec<usize> = (0..cases.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(took(&cases[i].0)));
    let ran = parallel::parallel(&order, |&i| {
        let started = std::time::Instant::now();
        let (verdict, detail) = verdict(&cases[i].1);
        (verdict, detail, started.elapsed())
    });
    let mut verdicts: Vec<_> = order.into_iter().zip(ran).collect();
    verdicts.sort_by_key(|(i, _)| *i);
    // A listed pass that timed out under a full machine's load gets one run alone, so a
    // load timeout is not a lost pass; a real timeout times out again.
    let passing =
        std::fs::read_to_string(root.join("crates/cpl-next/tests/passing.txt")).unwrap_or_default();
    let passing: std::collections::BTreeSet<&str> = passing.lines().map(str::trim).collect();
    for (i, v) in &mut verdicts {
        if v.0 == "timeout" && passing.contains(cases[*i].0.as_str()) {
            let started = std::time::Instant::now();
            let (verdict, detail) = verdict(&cases[*i].1);
            *v = (verdict, detail, v.2 + started.elapsed());
        }
    }
    verdicts.into_iter().map(|(_, v)| v).collect()
}

/// The harness runs cases on every core; a sample run one at a time gives the same verdicts.
/// What a worker left for one case: each file it wrote in the case's directory.
fn worker_files(w: &mut Worker, path: &Path, dir: &Path) -> BTreeMap<String, String> {
    std::fs::create_dir_all(dir).expect("the case directory is made");
    std::fs::write(dir.join("in"), b"").expect("the input is written");
    assert!(
        w.finished(path, dir),
        "{} finishes in the worker",
        path.display()
    );
    let files = std::fs::read_dir(dir)
        .expect("the case directory reads")
        .map(|e| e.unwrap().path());
    let out = files
        .map(|f| {
            (
                f.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read_to_string(&f).unwrap_or_default(),
            )
        })
        .collect();
    let _ = std::fs::remove_dir_all(dir);
    out
}

/// A case gives the same in a worker that has compiled others before it as in a worker of its
/// own, so nothing one case leaves (a query, a cache, an instance) reaches the next: lane D
/// saw 120 cases fail only inside the `--all` worker.
#[test]
fn a_case_in_the_worker_gives_what_it_gives_alone() {
    let cases: Vec<_> = all_cases()
        .into_iter()
        .filter(|(id, _)| id.starts_with("core-07/") || id.starts_with("std-heap/"))
        .step_by(7)
        .take(12)
        .collect();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("alone")
        .join(std::process::id().to_string());
    let alone: Vec<_> = std::thread::scope(|s| {
        let runs: Vec<_> = (cases.iter().enumerate())
            .map(|(i, (_, path))| {
                let dir = root.join(format!("alone-{i}"));
                s.spawn(move || worker_files(&mut Worker::new(), path, &dir))
            })
            .collect();
        runs.into_iter().map(|r| r.join().unwrap()).collect()
    });
    let mut shared = Worker::new();
    let mut differ = vec![];
    for (i, ((id, path), alone)) in cases.iter().zip(&alone).enumerate() {
        let inside = worker_files(&mut shared, path, &root.join(format!("shared-{i}")));
        if &inside != alone {
            let names: Vec<_> = (alone.keys().chain(inside.keys()))
                .filter(|k| inside.get(*k) != alone.get(*k))
                .collect();
            differ.push(format!("{id}: {names:?}"));
        }
    }
    assert!(
        differ.is_empty(),
        "cases the worker gives otherwise than alone:\n{}",
        differ.join("\n")
    );
}

#[test]
fn a_parallel_run_gives_what_a_serial_one_does() {
    let cases = all_cases();
    let sample: Vec<_> = cases.iter().step_by(cases.len() / 24 + 1).collect();
    let parallel = parallel::parallel(&sample, |(_, p)| verdict(p));
    let serial: Vec<_> = sample.iter().map(|(_, p)| verdict(p)).collect();
    assert_eq!(parallel, serial);
}

#[test]
fn every_case_is_where_the_lists_say() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = here.parent().and_then(Path::parent).unwrap();
    let mut cases = all_cases();
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
    let shard = cpl_env::NEXT_SHARD.text().map(|s| shard_spec(&s));
    let mine = |id: &str| shard.is_none_or(|(i, n)| in_shard(id, i, n));
    cases.retain(|(id, _)| mine(id));
    let verdicts = longest_first(root, &cases);

    let (mut passing, mut wrong, mut refused) = (String::new(), String::new(), String::new());
    let mut masked = String::new();
    for ((id, _), (verdict, detail, _)) in cases.iter().zip(&verdicts) {
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
    // Errors only lowering finds: the checker passed what lowering then refuses (suite 5).
    let lowered: String = (cases.iter().zip(&verdicts))
        .filter(|(_, (_, detail, _))| detail.starts_with("lower:"))
        .map(|((id, _), _)| format!("{id}\n"))
        .collect();
    // Instances holding an error type: the checker should have refused or resolved them.
    let typed = ERROR_TYPED.lock().unwrap().clone();
    let error_typed: String = (cases.iter())
        .filter(|(_, p)| typed.contains(&fnv(&std::fs::read(p).unwrap_or_default())))
        .map(|(id, _)| {
            format!(
                "{id}
"
            )
        })
        .collect();
    let found = root.join(match (&only, shard) {
        (Some(_), _) => "target/cpl-next/only".to_string(),
        (None, Some((i, n))) => format!("target/cpl-next/shard-{i}-of-{n}"),
        (None, None) => "target/cpl-next".to_string(),
    });
    std::fs::create_dir_all(&found).unwrap();
    let queue = queue(&refused);
    let mut times: Vec<(&str, u128)> = (cases.iter().zip(&verdicts))
        .map(|((id, _), (_, _, took))| (id.as_str(), took.as_millis()))
        .collect();
    times.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let times: String = times
        .iter()
        .map(|(id, ms)| format!("{id}\t{ms}\n"))
        .collect();
    for (name, text) in [
        ("times.tsv", &times),
        ("passing.txt", &passing),
        ("wrong.tsv", &wrong),
        ("refused.tsv", &refused),
        ("masked.txt", &masked),
        ("lowered.txt", &lowered),
        ("error-typed.txt", &error_typed),
        ("queue.txt", &queue),
    ] {
        std::fs::write(found.join(name), text).unwrap();
    }

    let (listed_passing, listed_wrong, listed_masked) = (
        list(&here.join("tests/passing.txt")),
        list(&here.join("tests/wrong.tsv")),
        list(&here.join("tests/masked.txt")),
    );
    // One run writes the lists, so a case is in one at most; an id in two is a hand edit or a
    // merge, and the run would answer only one of them.
    let twice: Vec<&str> = (listed_passing.keys())
        .filter(|id| listed_wrong.contains_key(*id) || listed_masked.contains_key(*id))
        .chain(
            listed_wrong
                .keys()
                .filter(|id| listed_masked.contains_key(*id)),
        )
        .map(String::as_str)
        .collect();
    assert!(
        twice.is_empty(),
        "listed in two of passing.txt, wrong.tsv and masked.txt (copy target/cpl-next/ over them):\n{}",
        twice.join("\n")
    );
    let mut off = Vec::new();
    let mut seen = Vec::new();
    let mut freed = 0;
    for ((id, _), (verdict, detail, _)) in cases.iter().zip(&verdicts) {
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
    let listed_lowered = list(&here.join("tests/lowered.txt"));
    for id in lowered
        .lines()
        .filter(|id| !listed_lowered.contains_key(*id))
    {
        off.push(format!(
            "{id}: an error only lowering finds: add it to tests/lowered.txt"
        ));
    }
    let listed_error_typed = list(&here.join("tests/error-typed.txt"));
    for id in (error_typed.lines()).filter(|id| !listed_error_typed.contains_key(*id)) {
        off.push(format!(
            "{id}: links an instance holding an error type, which the checker should have refused or resolved"
        ));
    }
    if (listed_error_typed.keys()).any(|id| only.is_none() && !error_typed.lines().any(|l| l == id))
    {
        println!("cases in tests/error-typed.txt hold no error type now: copy target/cpl-next/error-typed.txt over it");
    }
    for id in listed_passing
        .keys()
        .chain(listed_wrong.keys())
        .chain(listed_masked.keys())
        .filter(|id| only.is_none() && mine(id) && !seen.contains(&id.as_str()))
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
    if listed_lowered
        .keys()
        .any(|id| only.is_none() && mine(id) && !lowered.lines().any(|l| l == id))
    {
        println!("cases in tests/lowered.txt are caught before lowering now: copy target/cpl-next/lowered.txt over it");
    }
    if freed > 0 {
        println!("{freed} cases in tests/masked.txt no longer hide a verdict: copy target/cpl-next/masked.txt over it");
    }
    println!("refused, by cause (all in target/cpl-next/queue.txt):");
    queue.lines().take(10).for_each(|l| println!("  {l}"));
    // One row per misplaced case, so a land onto a main whose lists are already stale is
    // refused for the rows it adds, not let through as "red at the base too" (new_reds.py).
    off.iter().for_each(|l| println!("{}", finding(l)));
    assert!(
        off.is_empty(),
        "{} of {} cases are not where the lists say (what was found is in {}):\n{}",
        off.len(),
        cases.len(),
        found.display(),
        off.join("\n")
    );
}

/// `i/n` of `CPL_NEXT_SHARD`, 1 <= i <= n.
fn shard_spec(spec: &str) -> (u64, u64) {
    let parsed = spec
        .split_once('/')
        .and_then(|(i, n)| Some((i.trim().parse().ok()?, n.trim().parse().ok()?)));
    match parsed {
        Some((i, n)) if 1 <= i && i <= n => (i, n),
        _ => panic!("CPL_NEXT_SHARD={spec:?}: want i/n with 1 <= i <= n"),
    }
}

/// Whether case `id` is in shard `i` of `n`: by a hash of the id alone, so a listed id that
/// names no case still falls in exactly one shard, which reports it.
fn in_shard(id: &str, i: u64, n: u64) -> bool {
    let hash = id.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    hash % n == i - 1
}

#[test]
fn every_id_is_in_exactly_one_shard() {
    for n in [1, 2, 8] {
        for id in ["core-01/double-index.cp", "next/n_a.cp", "std-seq/x.cp", ""] {
            assert_eq!(
                (1..=n).filter(|&i| in_shard(id, i, n)).count(),
                1,
                "{id} in {n}"
            );
        }
    }
    assert_eq!(shard_spec("3/8"), (3, 8));
    for bad in ["0/8", "9/8", "8", "a/b"] {
        assert!(
            std::panic::catch_unwind(|| shard_spec(bad)).is_err(),
            "{bad}"
        );
    }
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

/// Every `@fast` fn in std means its CP body; its op is only a faster way to compute it. So
/// each is called on the edge words and on seeded random ones, and the reference interpreter
/// (the body), the engine and the C++ (the op), and the engine and the C++ under `--bodies`
/// all print the same. A `@fast` fn this cannot call is a finding: `fast_unchecked` in the
/// debt ratchet counts them by the same rule.
#[test]
fn fast_ops_agree_with_their_bodies() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (fns, uncovered) = fast_fns(&here.join("../../std"));
    for u in &uncovered {
        println!("cpl-finding: fast_ops_agree_with_their_bodies {u} +1");
    }
    assert!(
        uncovered.is_empty(),
        "@fast fns no input reaches: {uncovered:?}"
    );
    assert!(!fns.is_empty(), "std has no @fast fn: the scan is broken");
    let dir = here.join("../../target/cpl-next/scratch/fast-ops");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fast.cp");
    std::fs::write(&path, fast_program(&fns)).unwrap();
    let body = next(&path, &[], b"", LIMIT);
    assert_eq!(body.status, Some(0), "{}{}", body.stdout, body.stderr);
    let ints = fns.iter().filter(|f| f.at == "Int").count();
    let lines = body.stdout.lines().count();
    assert_eq!(lines, fns.len() * FAST_WORDS - ints, "one line per call");
    let agree = |what: &str, ran: Ran| {
        assert!(
            ran.status == Some(0) && ran.stdout == body.stdout,
            "{what}: exit {:?}, first line apart: {:?}",
            ran.status,
            (ran.stdout.lines().zip(body.stdout.lines()))
                .enumerate()
                .find(|(_, (a, b))| a != b)
        );
    };
    for bodies in [&[][..], &["--bodies"]] {
        let flags = |f: &'static str| [&[f][..], bodies].concat();
        agree(
            &format!("engine {bodies:?}"),
            next(&path, &flags("--engine"), b"", LIMIT),
        );
        let cpp = next(&path, &flags("--cpp"), b"", LIMIT).stdout;
        let bin = cxx::build(cpp.as_bytes(), &[&CXX_FLAGS[..], &["c++"]].concat())
            .unwrap_or_else(|e| panic!("C++ {bodies:?}: {e}"))
            .0;
        agree(&format!("C++ {bodies:?}"), run_kept(&bin, b"", LIMIT));
    }
}

/// A named lint is a lint, as an `L` code is; an error or trap code is not
/// (lints/std-declares-idiom-lints).
#[test]
fn a_named_lint_is_a_lint() {
    assert!(is_lint("prefix_totals_list") && is_lint("L0101"));
    assert!(!is_lint("E0902") && !is_lint("R0301") && !is_lint("W0101"));
}

/// A case's own `cp.toml`, written by `// file:`, defines its judge; without one only a
/// built-in judge resolves (tools-judge/check-memory-over-the-judges-limit).
#[test]
fn a_case_judge_comes_from_its_own_cp_toml() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = here.join("../../target/cpl-next/scratch/judge-of-a-case");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let case = dir.join("case.cp");
    std::fs::write(&case, "// expect: exit 1\n// judge: small\nprint(1)\n").unwrap();
    assert_eq!(judge(&case), None, "`small` is no built-in judge");
    std::fs::write(
        dir.join("cp.toml"),
        "[judge.small]\nextends = \"generic\"\nmemory_mb = 1\n",
    )
    .unwrap();
    assert_eq!(judge(&case).as_deref(), Some("small=1"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Without `--trace` a trap names the line of the operation that trapped on every runner,
/// also when that operation sits inside a `return`, a condition or a larger expression.
#[test]
fn a_trap_names_its_line_without_trace() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = here.join("../../target/cpl-next/scratch/trap-lines");
    std::fs::create_dir_all(&dir).unwrap();
    let bodies = [
        (
            "return",
            "    if n > 5:\n        return 0\n    return 10 / n\n",
        ),
        (
            "cond",
            "    if 10 / n > 1:\n        return 1\n    return 2\n",
        ),
        ("nested", "    let k = 3\n    return k + 10 / n\n"),
    ];
    for (name, body) in bodies {
        let path = dir.join(format!("{name}.cp"));
        std::fs::write(&path, format!("fn f(n: Int) -> Int:\n{body}print(f(0))\n")).unwrap();
        let line = body.lines().position(|l| l.contains("10 / n")).unwrap() + 2;
        let want = format!("trap: R0203: line {line}: division by zero");
        let cpp = next(&path, &["--cpp"], b"", LIMIT).stdout;
        let bin = cxx::build(cpp.as_bytes(), &[&CXX_FLAGS[..], &["c++"]].concat())
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .0;
        for (what, ran) in [
            ("reference", next(&path, &[], b"", LIMIT)),
            ("engine", next(&path, &["--engine"], b"", LIMIT)),
            ("C++", run_kept(&bin, b"", LIMIT)),
        ] {
            assert_eq!(trap_line(&ran.stderr), want, "{name} on {what}");
        }
    }
}

/// A traced C++ run writes its whole trace, past its buffer, before a trap's line and at exit.
#[test]
fn a_traced_cpp_run_writes_its_whole_trace() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = here.join("../../target/cpl-next/scratch/trace-flush");
    std::fs::create_dir_all(&dir).unwrap();
    let ends = [("exits", "0"), ("traps", "10 / n")];
    for (name, end) in ends {
        let path = dir.join(format!("{name}.cp"));
        let src =
            format!("fn f(n: Int) -> Int = if n == 0: {end} else: f(n - 1) + 1\nprint(f(5000))\n");
        std::fs::write(&path, src).unwrap();
        let cpp = next(&path, &["--cpp", "--trace"], b"", LIMIT).stdout;
        let bin = cxx::build(cpp.as_bytes(), &[&CXX_FLAGS[..], &["c++"]].concat())
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .0;
        let (want, got) = (
            next(&path, &["--trace"], b"", LIMIT),
            run_kept(&bin, b"", LIMIT),
        );
        assert!(
            got.stderr.len() > 1 << 16,
            "{name}: the trace fits its buffer"
        );
        // The call stack is the reference's own line after a trap; the C++ has none.
        let trace = want
            .stderr
            .lines()
            .filter(|l| !l.starts_with("call stack: "));
        let want_err: String = trace.map(|l| format!("{l}\n")).collect();
        assert_eq!((got.stdout, got.stderr), (want.stdout, want_err), "{name}");
    }
}

/// Every case `tests/smoke.txt` lists, a sample of unions, closures, lent slots and traced
/// instances, gives traced C++ that g++ accepts: a commit-tier check, quick enough to run on
/// every commit, that the C++ backend still builds what the listed passes cover.
#[test]
fn the_smoke_sample_gives_cpp_gxx_accepts() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ids = std::fs::read_to_string(here.join("tests/smoke.txt")).unwrap();
    let ids: Vec<&str> = (ids.lines().map(str::trim))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let cases: BTreeMap<String, PathBuf> = all_cases().into_iter().collect();
    let dir = here.join("../../target/cpl-next/scratch/smoke");
    std::fs::create_dir_all(&dir).unwrap();
    let bad = parallel::parallel(&ids, |id| {
        let path = cases
            .get(*id)
            .unwrap_or_else(|| panic!("{id}: no such case"));
        let ran = next(path, &["--cpp", "--trace"], b"", LIMIT);
        if ran.status != Some(0) {
            return Some(format!(
                "{id}: not accepted, so not a sample: {}",
                ran.stdout.trim()
            ));
        }
        let cpp = ran.stdout;
        let file = dir.join(format!("{}.cpp", id.replace('/', "_")));
        std::fs::write(&file, cpp).unwrap();
        let out = Command::new("g++")
            .args(["-std=c++17", "-fsyntax-only", "-w"])
            .arg(&file)
            .output()
            .expect("g++ runs");
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().find(|l| l.contains("error")).unwrap_or("");
        (!out.status.success()).then(|| format!("{id}: {first}"))
    });
    let bad: Vec<String> = bad.into_iter().flatten().collect();
    assert!(bad.is_empty(), "g++ rejects:\n{}", bad.join("\n"));
}

/// How many words each `@fast` fn is called on: the edge words, then seeded random ones.
const FAST_WORDS: usize = 256;

/// The words a `@fast` fn is called on: 0, every power of two and its neighbours, the
/// all-ones word, then random words from a fixed seed.
fn fast_words() -> Vec<u64> {
    let mut w = vec![0, u64::MAX];
    for k in 0..64 {
        let p = 1u64 << k;
        w.extend([p, p - 1, p.wrapping_add(1)]);
    }
    w.sort_unstable();
    w.dedup();
    let mut s = 0x9e37_79b9_7f4a_7c15u64;
    while w.len() < FAST_WORDS {
        s = s.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        w.push(z ^ (z >> 31));
    }
    w
}

/// A `@fast` fn of std: its module, its name and the type it is called at.
struct FastFn {
    module: String,
    name: String,
    at: &'static str,
}

/// The types a `@fast` fn's word can be written in: an `Int` as its two's complement.
const FAST_TYPES: [&str; 2] = ["Int", "U64"];

/// Every `@fast` fn under `std`, once per type it is called at, and the ones no input
/// reaches (`path:line name`): a fn of one parameter whose type is `Int`, `U64` or a `W:
/// Word` is called at each; anything else is not.
fn fast_fns(std_dir: &Path) -> (Vec<FastFn>, Vec<String>) {
    let mut files = Vec::new();
    let mut todo = vec![std_dir.to_path_buf()];
    while let Some(d) = todo.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            match p.extension() {
                _ if p.is_dir() => todo.push(p),
                Some(x) if x == "cp" => files.push(p),
                _ => {}
            }
        }
    }
    files.sort();
    let src: Vec<(PathBuf, String)> = (files.into_iter())
        .map(|p| (std::fs::read_to_string(&p).unwrap(), p))
        .map(|(s, p)| (p, s))
        .collect();
    let words: Vec<&str> = (src.iter())
        .flat_map(|(_, s)| s.lines())
        .filter_map(|l| l.strip_prefix("impl Word for ")?.strip_suffix(':'))
        .collect();
    let (mut fns, mut uncovered) = (Vec::new(), Vec::new());
    for (p, s) in &src {
        let rel = p.strip_prefix(std_dir).unwrap().with_extension("");
        let module = format!("std.{}", rel.to_string_lossy().replace('/', "."));
        let lines: Vec<&str> = s.lines().collect();
        for (i, l) in lines.iter().enumerate() {
            if !l.starts_with("@fast(") {
                continue;
            }
            let Some(sig) = lines[i..].iter().find_map(|l| l.strip_prefix("pub fn ")) else {
                uncovered.push(format!("{}:{} no fn", p.display(), i + 1));
                continue;
            };
            let name = sig.split(['<', '(']).next().unwrap_or(sig).to_string();
            let generic = (sig.split_once('<').filter(|(n, _)| !n.contains('(')))
                .and_then(|(_, g)| g.split_once('>'))
                .and_then(|(g, _)| g.strip_suffix(": Word"));
            let param = (sig.split_once('(').and_then(|(_, r)| r.split_once(')')))
                .and_then(|(ps, _)| ps.split_once(": "))
                .map(|(_, t)| t)
                .filter(|t| !t.contains(','));
            let at: Vec<&str> = match param {
                Some(t) if Some(t) == generic => words.clone(),
                Some(t) => vec![t],
                None => vec![],
            };
            let known: Vec<&'static str> = (at.iter())
                .filter_map(|t| FAST_TYPES.iter().find(|k| *k == t).copied())
                .collect();
            if at.is_empty() || known.len() < at.len() {
                uncovered.push(format!("{}:{} {name}", p.display(), i + 1));
                continue;
            }
            for at in known {
                fns.push(FastFn {
                    module: module.clone(),
                    name: name.clone(),
                    at,
                });
            }
        }
    }
    (fns, uncovered)
}

/// A program that calls each `@fast` fn on every [`fast_words`] word and prints the results.
fn fast_program(fns: &[FastFn]) -> String {
    let words = fast_words();
    let int = |w: u64| (w as i64).to_string();
    let u64 = |w: u64| format!("(U64({}) << 32) | U64({})", w >> 32, w & 0xffff_ffff);
    // `Int` has no value for the word `1 << 63`, which is `?Int`'s `None` (decision 49).
    let list = |f: &dyn Fn(u64) -> String, int: bool| {
        let xs: Vec<String> = (words.iter())
            .filter(|&&w| !int || w != 1 << 63)
            .map(|&w| f(w))
            .collect();
        format!("[{}]", xs.join(", "))
    };
    let mut p = String::new();
    for f in fns {
        p += &format!("use {}.{{{}}}\n", f.module, f.name);
    }
    p += &format!("let ints = {}\n", list(&int, true));
    p += &format!("let words = {}\n", list(&u64, false));
    for f in fns {
        let xs = if f.at == "Int" { "ints" } else { "words" };
        p += &format!("for x in {xs}: print({}(x))\n", f.name);
    }
    p
}
