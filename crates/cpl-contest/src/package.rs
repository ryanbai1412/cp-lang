//! A problem package's cases from its `tests.toml` (problem-packages/R9).
//! Each `[[case]]` is an `input` written inline, an `input_file` beside it
//! in the package (a statement sample too big to inline), or what
//! `gen.py seed size` or `gen.py args..` prints. A case's answer is what the accepted C++
//! prints on it; an interactive package's case is the interactor's hidden
//! data and has none. `cpl test`, `scripts/corpus.py make` and the tests
//! all read the cases made here, in `target/corpus-tests/<id>/`.

use std::hash::{DefaultHasher, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// How long the accepted C++ may take on one case.
const REFERENCE_TIME: Duration = Duration::from_secs(20);
/// How long `gen.py` may take to print one case.
const GEN_TIME: Duration = Duration::from_secs(60);
/// Changes when the way cases are made does, so older ones are made again.
const MAKER: &str = "cpl-cases 1";

/// One `[[case]]` of a tests.toml.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Case {
    /// The input, written out.
    Input(String),
    /// The input in a file, its path relative to the package.
    File(PathBuf),
    /// `gen.py`'s arguments: `seed size`, or a case's `args`.
    Gen(Vec<String>),
}

impl Case {
    /// Whether the case is the generator's largest (`args = ["--max"]`).
    pub fn is_max(&self) -> bool {
        matches!(self, Case::Gen(args) if args.iter().any(|a| a == "--max"))
    }
}

/// The cases a tests.toml lists, in order.
pub fn parse(text: &str) -> Result<Vec<Case>, String> {
    let mut cases = Vec::new();
    let mut fields: Option<Fields> = None;
    let mut lines = text.lines().enumerate();
    while let Some((at, line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = at + 1;
        if line == "[[case]]" {
            if let Some(done) = fields.take() {
                cases.push(done.case(cases.len() + 1)?);
            }
            fields = Some(Fields::default());
            continue;
        }
        let Some(case) = fields.as_mut() else {
            return Err(format!("line {at}: `{line}` comes before any `[[case]]`"));
        };
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("line {at}: `{line}` is not `key = value`"));
        };
        let value = value.trim();
        let value = match value.strip_prefix("\"\"\"") {
            Some(first) => multi_line(first, &mut lines.by_ref().map(|(_, l)| l))
                .ok_or_else(|| format!("line {at}: a `\"\"\"` string that never ends"))?,
            None => Value::of(value).ok_or_else(|| format!("line {at}: can't read `{value}`"))?,
        };
        case.set(key.trim(), value)
            .map_err(|why| format!("line {at}: {why}"))?;
    }
    if let Some(done) = fields {
        cases.push(done.case(cases.len() + 1)?);
    }
    Ok(cases)
}

#[derive(Default)]
struct Fields {
    input: Option<String>,
    input_file: Option<String>,
    seed: Option<i64>,
    size: Option<i64>,
    args: Option<Vec<String>>,
}

impl Fields {
    fn set(&mut self, key: &str, value: Value) -> Result<(), String> {
        match (key, value) {
            ("input", Value::Str(s)) => self.input = Some(s),
            ("input_file", Value::Str(s)) => self.input_file = Some(s),
            ("seed", Value::Int(n)) => self.seed = Some(n),
            ("size", Value::Int(n)) => self.size = Some(n),
            ("args", Value::List(items)) => self.args = Some(items),
            // A label for whoever reads the file.
            ("name", Value::Str(_)) => {}
            (key, _) => {
                return Err(format!(
                    "`{key}` is not `input`, `input_file`, `seed`, `size`, `args` or `name` of its type"
                ))
            }
        }
        Ok(())
    }

    fn case(self, n: usize) -> Result<Case, String> {
        match self {
            Fields {
                input: Some(input),
                input_file: None,
                seed: None,
                size: None,
                args: None,
            } => Ok(Case::Input(if input.ends_with('\n') {
                input
            } else {
                input + "\n"
            })),
            Fields {
                input: Some(_),
                input_file: Some(_),
                ..
            } => Err(format!(
                "case {n:02}: give `input` or `input_file`, not both"
            )),
            Fields {
                input: None,
                input_file: Some(file),
                seed: None,
                size: None,
                args: None,
            } => Ok(Case::File(PathBuf::from(file))),
            Fields {
                input: None,
                input_file: None,
                seed: Some(seed),
                size: Some(size),
                args: None,
            } if size >= 1 => Ok(Case::Gen(vec![seed.to_string(), size.to_string()])),
            Fields {
                size: Some(size), ..
            } if size < 1 => Err(format!(
                "case {n:02}: size {size}, but `gen.py seed size` takes a size of at least 1"
            )),
            Fields {
                input: None,
                input_file: None,
                seed: None,
                size: None,
                args: Some(args),
            } => Ok(Case::Gen(args)),
            _ => Err(format!(
                "case {n:02}: give `input` or `input_file`, or `seed` and `size`, or `args`, and only one"
            )),
        }
    }
}

enum Value {
    Str(String),
    Int(i64),
    List(Vec<String>),
}

impl Value {
    fn of(text: &str) -> Option<Value> {
        if let Some(rest) = text.strip_prefix('"') {
            let (s, rest) = basic(rest)?;
            return rest.trim().is_empty().then_some(Value::Str(s));
        }
        if let Some(rest) = text.strip_prefix('[') {
            let mut items = Vec::new();
            let mut rest = rest.trim_start();
            loop {
                if let Some(after) = rest.strip_prefix(']') {
                    return after.trim().is_empty().then_some(Value::List(items));
                }
                if let Some(quoted) = rest.strip_prefix('"') {
                    let (s, after) = basic(quoted)?;
                    items.push(s);
                    rest = after;
                } else {
                    let end = rest.find([',', ']'])?;
                    items.push(rest[..end].trim().parse::<i64>().ok()?.to_string());
                    rest = &rest[end..];
                }
                rest = rest.trim_start();
                rest = rest.strip_prefix(',').unwrap_or(rest).trim_start();
            }
        }
        text.replace('_', "").parse().ok().map(Value::Int)
    }
}

/// A basic string's text after its opening `"`, and what follows its
/// closing one.
fn basic(text: &str) -> Option<(String, &str)> {
    let mut out = String::new();
    let mut chars = text.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((out, &text[i + 1..])),
            '\\' => out.push(escape(&mut chars.by_ref().map(|(_, c)| c))?),
            c => out.push(c),
        }
    }
    None
}

/// A `"""` string from the text after its opening quotes: a newline right
/// after them is not part of it.
fn multi_line(first: &str, rest: &mut dyn Iterator<Item = &str>) -> Option<Value> {
    let mut raw = String::new();
    let mut line = first.to_string();
    let mut started = !first.is_empty();
    loop {
        if let Some(end) = line.find("\"\"\"") {
            raw.push_str(&line[..end]);
            if !line[end + 3..].trim().is_empty() {
                return None;
            }
            break;
        }
        if started {
            raw.push_str(&line);
            raw.push('\n');
        }
        started = true;
        line = rest.next()?.to_string();
    }
    let mut out = String::new();
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push(escape(&mut chars)?),
            c => out.push(c),
        }
    }
    Some(Value::Str(out))
}

fn escape(chars: &mut dyn Iterator<Item = char>) -> Option<char> {
    Some(match chars.next()? {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '"' => '"',
        '\\' => '\\',
        'u' => {
            let hex: String = chars.take(4).collect();
            char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?
        }
        _ => return None,
    })
}

/// The repo a package lives in: the folder that holds its `corpus/`.
fn repo_of(package: &Path) -> PathBuf {
    package
        .ancestors()
        .find(|dir| dir.join("corpus").is_dir())
        .unwrap_or(package)
        .to_path_buf()
}

/// Where `package`'s cases are made: `target/corpus-tests/<id>/` in its repo.
pub fn cases_dir(package: &Path) -> PathBuf {
    let package = package
        .canonicalize()
        .unwrap_or_else(|_| package.to_path_buf());
    let id = package.file_name().unwrap_or_default();
    repo_of(&package)
        .join("target")
        .join("corpus-tests")
        .join(id)
}

/// The accepted C++ a package's answers come from: `orig.cpp`, else the
/// first `.cpp` submission `meta.json` lists as AC.
pub fn reference(package: &Path) -> Option<PathBuf> {
    let orig = package.join("orig.cpp");
    if orig.is_file() {
        return Some(orig);
    }
    let meta = std::fs::read_to_string(package.join("meta.json")).ok()?;
    let meta = cpl_json::parse(&meta).ok()?;
    let root = repo_of(package);
    meta.at(&["submissions"])
        .as_array()?
        .iter()
        .filter(|s| s.at(&["result"]).as_str() == Some("AC"))
        .filter_map(|s| s.at(&["file"]).as_str())
        .filter(|file| file.ends_with(".cpp"))
        .map(|file| root.join(file))
        .find(|path| path.is_file())
}

/// The file an `input_file` case names, `path` relative to `package`:
/// an error when it is missing or, links and `..` followed, outside the package.
pub fn input_file(package: &Path, path: &Path) -> Result<PathBuf, String> {
    let package = package
        .canonicalize()
        .map_err(|e| format!("can't find {}: {e}", package.display()))?;
    let file = package
        .join(path)
        .canonicalize()
        .map_err(|e| format!("input_file {}: {e}", path.display()))?;
    if !file.starts_with(&package) {
        return Err(format!(
            "input_file {} is outside the package",
            path.display()
        ));
    }
    if !file.is_file() {
        return Err(format!("input_file {} is not a file", path.display()));
    }
    Ok(file)
}

/// Where one case's input comes from, its `input_file` already checked.
enum Source<'a> {
    Inline(&'a str),
    File(PathBuf),
    Gen(&'a [String]),
}

/// Each case's [`Source`]; a bad `input_file` fails naming its case.
fn sources<'a>(package: &Path, cases: &'a [Case]) -> Result<Vec<Source<'a>>, String> {
    let source = |(i, case): (usize, &'a Case)| match case {
        Case::Input(text) => Ok(Source::Inline(text)),
        Case::File(path) => input_file(package, path)
            .map(Source::File)
            .map_err(|e| format!("case {:02}: {e}", i + 1)),
        Case::Gen(args) => Ok(Source::Gen(args)),
    };
    cases.iter().enumerate().map(source).collect()
}

/// Makes `package`'s cases in `out` unless the ones there are current:
/// `NN.in`, `NN.ans` from the accepted C++ (none for an interactive
/// package), and an empty `NN.max` beside the generator's largest.
pub fn make(package: &Path, out: &Path) -> Result<(), String> {
    make_within(package, out, REFERENCE_TIME)
}

/// [`make`], with the accepted C++ given `budget` on each case: one that
/// runs past it fails naming the case.
pub fn make_within(package: &Path, out: &Path, budget: Duration) -> Result<(), String> {
    let package = &package
        .canonicalize()
        .map_err(|e| format!("can't find {}: {e}", package.display()))?;
    let toml = package.join("tests.toml");
    let text = std::fs::read_to_string(&toml)
        .map_err(|e| format!("can't read {}: {e}", toml.display()))?;
    let cases = parse(&text).map_err(|e| format!("{}: {e}", toml.display()))?;
    let interactive = package.join("interactor.py").is_file();
    let reference = match interactive {
        true => None,
        false => Some(reference(package).ok_or_else(|| {
            format!(
                "{} has no accepted C++ (orig.cpp or an AC submission) to make answers",
                package.display()
            )
        })?),
    };
    let gen = package.join("gen.py");
    let sources = sources(package, &cases).map_err(|e| format!("{}: {e}", toml.display()))?;
    let mut stamp = DefaultHasher::new();
    stamp.write(MAKER.as_bytes());
    for file in [Some(&toml), Some(&gen), reference.as_ref()]
        .into_iter()
        .flatten()
        .chain(sources.iter().filter_map(|s| match s {
            Source::File(file) => Some(file),
            _ => None,
        }))
    {
        stamp.write(&std::fs::read(file).unwrap_or_default());
        stamp.write_u8(0);
    }
    let stamp = format!("{:016x}\n", stamp.finish());
    let current =
        |dir: &Path| std::fs::read_to_string(dir.join(".stamp")).ok() == Some(stamp.clone());
    if current(out) {
        return Ok(());
    }
    let root = repo_of(package);
    let exe = match &reference {
        Some(src) => Some(built(src, package)?),
        None => None,
    };
    let parent = out.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("can't create {}: {e}", parent.display()))?;
    // Made aside and renamed in, so a reader never sees half of them.
    let name = out.file_name().unwrap_or_default().to_string_lossy();
    static MAKERS: AtomicUsize = AtomicUsize::new(0);
    let maker = MAKERS.fetch_add(1, Ordering::Relaxed);
    let aside = parent.join(format!(".{name}.{}.{maker}", std::process::id()));
    let _ = std::fs::remove_dir_all(&aside);
    let made = (|| {
        std::fs::create_dir_all(&aside).map_err(|e| e.to_string())?;
        for (i, (case, source)) in cases.iter().zip(&sources).enumerate() {
            let n = format!("{:02}", i + 1);
            let mut input = match source {
                Source::Inline(text) => text.as_bytes().to_vec(),
                Source::File(file) => std::fs::read(file)
                    .map_err(|e| format!("can't read {}: {e}", file.display()))?,
                Source::Gen(args) => {
                    let mut cmd = Command::new("python3");
                    cmd.arg(&gen).args(*args).current_dir(&root);
                    match run(&mut cmd, b"", GEN_TIME, "gen.py")? {
                        Ran::Past => {
                            return Err(format!(
                                "gen.py {} runs past {} s on case {n}",
                                args.join(" "),
                                GEN_TIME.as_secs()
                            ))
                        }
                        Ran::Exited(Some(0), out) => out,
                        Ran::Exited(code, _) => {
                            return Err(format!(
                                "gen.py {} exits {} on case {n}",
                                args.join(" "),
                                exit_code(code)
                            ))
                        }
                    }
                }
            };
            input = lf(input);
            if !input.ends_with(b"\n") {
                input.push(b'\n');
            }
            write(&aside.join(format!("{n}.in")), &input)?;
            if case.is_max() {
                write(&aside.join(format!("{n}.max")), b"")?;
            }
            if let Some(exe) = &exe {
                match run(&mut judge_stack(exe), &input, budget, "the accepted C++")? {
                    Ran::Exited(Some(0), answer) => {
                        write(&aside.join(format!("{n}.ans")), &lf(answer))?
                    }
                    Ran::Past => {
                        return Err(format!(
                            "accepted C++ runs past {} s on case {n}",
                            budget.as_secs()
                        ))
                    }
                    Ran::Exited(code, _) => {
                        return Err(format!(
                            "accepted C++ exits {} on case {n}",
                            exit_code(code)
                        ))
                    }
                }
            }
        }
        write(&aside.join(".stamp"), stamp.as_bytes())?;
        // `out` is only ever renamed, never deleted in place, so it is absent
        // or a whole folder: a delete is not atomic, and a concurrent maker
        // met a folder still emptying (Windows can't rename onto it) or read
        // its stamp gone. A rename in fails only once another maker's cases
        // are in, so each turn either finds them current or moves them out.
        let old = parent.join(format!(".{name}.{}.{maker}.old", std::process::id()));
        let mut failed = None;
        for _ in 0..64 {
            if current(out) {
                return Ok(());
            }
            if std::fs::rename(out, &old).is_ok() {
                let _ = std::fs::remove_dir_all(&old);
            }
            match std::fs::rename(&aside, out) {
                Ok(()) => return Ok(()),
                Err(e) => failed = Some(e),
            }
        }
        match failed {
            Some(e) if !current(out) => {
                Err(format!("can't move the cases to {}: {e}", out.display()))
            }
            _ => Ok(()),
        }
    })();
    let _ = std::fs::remove_dir_all(&aside);
    made.map_err(|e| format!("{}: {e}", package.display()))
}

fn exit_code(code: Option<i32>) -> String {
    code.map_or("on a signal".to_string(), |c| c.to_string())
}

/// `bytes` with each run of `\r`s before a `\n` dropped: cases and answers
/// are LF on every OS, as the judge's program runs see them, so neither a
/// Windows `print`'s `\r\n` nor the `\r\r\n` a text-mode stdout makes of
/// a written `\r\n` reaches a saved case.
pub(crate) fn lf(bytes: Vec<u8>) -> Vec<u8> {
    if !bytes.contains(&b'\r') {
        return bytes;
    }
    let mut out = Vec::with_capacity(bytes.len());
    for (i, &b) in bytes.iter().enumerate() {
        let ends_line = bytes[i..].iter().find(|&&c| c != b'\r') == Some(&b'\n');
        if !(b == b'\r' && ends_line) {
            out.push(b);
        }
    }
    out
}

fn write(path: &Path, data: &[u8]) -> Result<(), String> {
    std::fs::write(path, data).map_err(|e| format!("can't write {}: {e}", path.display()))
}

/// `exe` run with the judges' stack, so a deep recursion in the accepted
/// C++ runs as it did there.
fn judge_stack(exe: &Path) -> Command {
    if cfg!(unix) {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "ulimit -s unlimited 2>/dev/null; exec \"$0\""])
            .arg(exe);
        cmd
    } else {
        Command::new(exe)
    }
}

/// How a run within a time budget ended.
enum Ran {
    /// The exit code (none on a signal) and the output.
    Exited(Option<i32>, Vec<u8>),
    /// Still running at the budget, so killed.
    Past,
}

/// Runs `cmd` on `input` for at most `time`.
fn run(cmd: &mut Command, input: &[u8], time: Duration, what: &str) -> Result<Ran, String> {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("can't run {what}: {e}"))?;
    let (Some(mut stdin), Some(mut stdout)) = (child.stdin.take(), child.stdout.take()) else {
        return Err(format!("{what} has no pipes"));
    };
    let input = input.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        let _ = stdout.read_to_end(&mut out);
        out
    });
    let mut exit = cpl_wait::on_exit(child);
    let status = exit.within(time).map_err(|e| format!("{what}: {e}"))?;
    let Some(status) = status else {
        let _ = exit.kill();
        return Ok(Ran::Past);
    };
    let _ = writer.join();
    let out = reader.join().unwrap_or_default();
    Ok(Ran::Exited(status.code(), out))
}

/// Accepted submissions were judged by g++: C++23 (deducing `this`,
/// `<ranges>`), GNU dialect and libstdc++'s extensions (pb_ds, rope). So
/// references build with g++ 14 or newer, statically linked so a toolchain
/// outside the system's runs them too.
const REFERENCE_GXX: &[&str] = &["g++-15", "g++-14", "g++"];
const REFERENCE_FLAGS: &[&str] = &[
    "-std=gnu++23",
    "-O2",
    "-w",
    "-static-libstdc++",
    "-static-libgcc",
];

/// The g++ 14+ that builds accepted C++, and its version: one on `PATH`,
/// else the one the repo's `scripts/gxx14.sh` fetches.
fn reference_cxx(package: &Path) -> Result<(PathBuf, String), String> {
    let version_14 = |gxx: &Path| {
        let out = Command::new(gxx).arg("-dumpfullversion").output().ok()?;
        let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let major: u32 = version.split('.').next()?.parse().ok()?;
        (out.status.success() && major >= 14).then_some(version)
    };
    for gxx in REFERENCE_GXX {
        if let Some(version) = version_14(Path::new(gxx)) {
            return Ok((PathBuf::from(gxx), version));
        }
    }
    let script = package
        .ancestors()
        .map(|dir| dir.join("scripts").join("gxx14.sh"))
        .find(|script| script.is_file());
    if let Some(script) = script.filter(|_| !cpl_env::GXX_FETCH.is("0")) {
        let fetched = Command::new("bash")
            .arg(&script)
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| format!("can't run {}: {e}", script.display()))?;
        let path = PathBuf::from(String::from_utf8_lossy(&fetched.stdout).trim());
        if fetched.status.success() {
            if let Some(version) = version_14(&path) {
                return Ok((path, version));
            }
        }
    }
    Err(format!(
        "no g++ 14 or newer on PATH (tried {}) and scripts/gxx14.sh could not fetch one; \
         accepted C++ needs the judges' C++23 g++ (README: Setup)",
        REFERENCE_GXX.join(", ")
    ))
}

/// The accepted C++ ([`reference`]) built with the reference g++: what a
/// package with no port is judged by.
pub fn reference_exe(package: &Path) -> Result<PathBuf, String> {
    let src = reference(package).ok_or_else(|| {
        format!(
            "{} has no accepted C++ (orig.cpp or an AC submission)",
            package.display()
        )
    })?;
    built(&src, package)
}

/// `src` built with the reference g++, cached by content and compiler in
/// `~/.cache/cpl/cxx/corpus/`.
fn built(src: &Path, package: &Path) -> Result<PathBuf, String> {
    let (gxx, version) = reference_cxx(package)?;
    let data = std::fs::read(src).map_err(|e| format!("can't read {}: {e}", src.display()))?;
    let mut key = DefaultHasher::new();
    key.write(&data);
    key.write(b"\0gcc ");
    key.write(version.as_bytes());
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let cache = home.join(".cache").join("cpl").join("cxx").join("corpus");
    std::fs::create_dir_all(&cache)
        .map_err(|e| format!("can't create {}: {e}", cache.display()))?;
    let exe = cache.join(format!("rs-{:016x}", key.finish()));
    if exe.is_file() {
        return Ok(exe);
    }
    // Built aside and renamed in: a parallel maker never runs half a binary.
    let tmp = exe.with_extension(std::process::id().to_string());
    let build = |extra: &[&str]| {
        Command::new(&gxx)
            .args(REFERENCE_FLAGS)
            .args(extra)
            .arg("-o")
            .arg(&tmp)
            .arg(src)
            .output()
            .map_err(|e| format!("can't run {}: {e}", gxx.display()))
    };
    let mut out = build(&[])?;
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    if !out.status.success() && stderr.contains("target specific option mismatch") {
        // `#pragma GCC target("avx2")` before the includes: the judges' CPUs have AVX2.
        out = build(&["-mavx2"])?;
    }
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let short: String = stderr.chars().take(200).collect();
        return Err(format!("c++: {short}"));
    }
    std::fs::rename(&tmp, &exe).map_err(|e| format!("can't move {}: {e}", tmp.display()))?;
    Ok(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tests_toml_lists_inline_generated_and_argument_cases() {
        let text = "# cases\n\n[[case]]\nname = \"sample\"\ninput = \"1 2\"\n\n[[case]]\ninput = \"\"\"\n3\n4 \\\"x\\\"\n\"\"\"\n\n[[case]]\nseed = 7\nsize = 4\n\n[[case]]\nargs = [\"--max\", \"free\"]\n";
        let cases = parse(text).unwrap();
        assert_eq!(
            cases,
            [
                Case::Input("1 2\n".into()),
                Case::Input("3\n4 \"x\"\n".into()),
                Case::Gen(vec!["7".into(), "4".into()]),
                Case::Gen(vec!["--max".into(), "free".into()]),
            ]
        );
        assert!(!cases[2].is_max() && cases[3].is_max());
    }

    #[test]
    fn a_case_that_is_two_kinds_or_too_small_says_which() {
        let both = parse("[[case]]\ninput = \"1\"\nseed = 1\nsize = 2\n").unwrap_err();
        assert!(
            both.contains("case 01") && both.contains("only one"),
            "{both}"
        );
        let small = parse("[[case]]\nseed = 1\nsize = 0\n").unwrap_err();
        assert!(small.contains("at least 1"), "{small}");
        let stray = parse("input = \"1\"\n").unwrap_err();
        assert!(stray.contains("before any `[[case]]`"), "{stray}");
        let file = parse("[[case]]\ninput_file = \"samples/1.in\"\n").unwrap();
        assert_eq!(file, [Case::File("samples/1.in".into())]);
        let both = parse("[[case]]\ninput = \"1\"\ninput_file = \"samples/1.in\"\n").unwrap_err();
        assert!(
            both.contains("case 01") && both.contains("not both"),
            "{both}"
        );
    }

    #[test]
    fn an_input_file_case_is_the_file_and_makes_again_when_it_changes() {
        let dir = package("file", "[[case]]\ninput_file = \"samples/1.in\"\n", "");
        std::fs::create_dir_all(dir.join("samples")).unwrap();
        std::fs::write(dir.join("samples/1.in"), "3\n1 2 3").unwrap();
        let out = cases_dir(&dir);
        make(&dir, &out).unwrap();
        assert_eq!(text(out.join("01.in")), "3\n1 2 3\n");
        std::fs::write(dir.join("samples/1.in"), "1\n4\n").unwrap();
        make(&dir, &out).unwrap();
        assert_eq!(text(out.join("01.in")), "1\n4\n");
        std::fs::remove_dir_all(dir.ancestors().nth(3).unwrap()).unwrap();
    }

    #[test]
    fn an_input_file_missing_or_outside_the_package_names_its_case() {
        let toml = "[[case]]\ninput = \"1\"\n\n[[case]]\ninput_file = \"samples/none.in\"\n";
        let dir = package("nofile", toml, "");
        let out = cases_dir(&dir);
        let missing = make(&dir, &out).unwrap_err();
        assert!(
            missing.contains("case 02: input_file samples/none.in"),
            "{missing}"
        );
        std::fs::write(dir.parent().unwrap().join("secret.in"), "1\n").unwrap();
        std::fs::write(
            dir.join("tests.toml"),
            "[[case]]\ninput_file = \"../secret.in\"\n",
        )
        .unwrap();
        let outside = make(&dir, &out).unwrap_err();
        assert!(
            outside.contains("case 01: input_file ../secret.in is outside the package"),
            "{outside}"
        );
        assert!(!out.exists());
        std::fs::remove_dir_all(dir.ancestors().nth(3).unwrap()).unwrap();
    }

    /// An interactive package (its cases have no answers, so no C++
    /// builds) whose tests.toml is `toml`, beside `gen`.
    fn package(name: &str, toml: &str, gen: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("cpl-package-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("corpus/problems").join("p");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("tests.toml"), toml).unwrap();
        std::fs::write(dir.join("gen.py"), gen).unwrap();
        std::fs::write(dir.join("interactor.py"), "").unwrap();
        dir
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn text(path: PathBuf) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    #[test]
    fn cases_are_made_once_and_again_when_the_list_changes() {
        let gen = "import sys\nprint(9 if sys.argv[1] == '--max' else sys.argv[1])\n";
        let toml = "[[case]]\ninput = \"1 5\"\n\n[[case]]\nseed = 7\nsize = 1\n\n[[case]]\nargs = [\"--max\"]\n";
        let dir = package("once", toml, gen);
        let out = cases_dir(&dir);
        assert!(out.ends_with("target/corpus-tests/p"), "{}", out.display());
        make(&dir, &out).unwrap();
        assert_eq!(names(&out), [".stamp", "01.in", "02.in", "03.in", "03.max"]);
        assert_eq!(text(out.join("01.in")), "1 5\n");
        assert_eq!(text(out.join("02.in")), "7\n");
        assert_eq!(text(out.join("03.in")), "9\n");
        std::fs::write(out.join("01.in"), "kept\n").unwrap();
        make(&dir, &out).unwrap();
        assert_eq!(text(out.join("01.in")), "kept\n");
        std::fs::write(dir.join("tests.toml"), "[[case]]\ninput = \"2\"\n").unwrap();
        make(&dir, &out).unwrap();
        assert_eq!(names(&out), [".stamp", "01.in"]);
        assert_eq!(text(out.join("01.in")), "2\n");
        std::fs::remove_dir_all(dir.ancestors().nth(3).unwrap()).unwrap();
    }

    #[test]
    fn cases_are_lf_whatever_wrote_them() {
        let gen = "import sys\nsys.stdout.write('7\\r\\n8\\r\\n')\n";
        let toml = "[[case]]\ninput = \"1\\r\\n2\"\n\n[[case]]\nseed = 7\nsize = 1\n";
        let dir = package("lf", toml, gen);
        let out = cases_dir(&dir);
        make(&dir, &out).unwrap();
        assert_eq!(text(out.join("01.in")), "1\n2\n");
        assert_eq!(text(out.join("02.in")), "7\n8\n");
        assert_eq!(lf(b"a\rb\r\n\r".to_vec()), b"a\rb\n\r");
        assert_eq!(lf(b"7\r\r\n8\r\r\n".to_vec()), b"7\n8\n");
        std::fs::remove_dir_all(dir.ancestors().nth(3).unwrap()).unwrap();
    }

    #[test]
    fn a_failed_make_keeps_the_last_cases_whole() {
        let dir = package(
            "failed",
            "[[case]]\ninput = \"a\"\n",
            "import sys\nsys.exit(1)\n",
        );
        let out = cases_dir(&dir);
        make(&dir, &out).unwrap();
        let toml = "[[case]]\ninput = \"b\"\n\n[[case]]\nseed = 1\nsize = 1\n";
        std::fs::write(dir.join("tests.toml"), toml).unwrap();
        let failed = make(&dir, &out).unwrap_err();
        assert!(failed.contains("gen.py 1 1 exits 1 on case 02"), "{failed}");
        assert_eq!(names(&out), [".stamp", "01.in"]);
        assert_eq!(text(out.join("01.in")), "a\n");
        assert_eq!(names(out.parent().unwrap()), ["p"]);
        std::fs::remove_dir_all(dir.ancestors().nth(3).unwrap()).unwrap();
    }

    #[test]
    fn concurrent_makes_of_one_package_all_succeed() {
        let toml: String = (0..200)
            .map(|i| format!("[[case]]\ninput = \"{i}\"\n\n"))
            .collect();
        let dir = package("concurrent", &toml, "");
        let out = cases_dir(&dir);
        std::thread::scope(|s| {
            let makers: Vec<_> = (0..8).map(|_| s.spawn(|| make(&dir, &out))).collect();
            for maker in makers {
                maker.join().unwrap().unwrap();
            }
        });
        let inputs = names(&out).iter().filter(|n| n.ends_with(".in")).count();
        assert_eq!(inputs, 200);
        std::fs::remove_dir_all(dir.ancestors().nth(3).unwrap()).unwrap();
    }

    #[test]
    fn every_corpus_tests_toml_reads() {
        let mut read = 0;
        for package in cpl_corpus::packages(&cpl_corpus::root()) {
            let toml = package.join("tests.toml");
            if let Ok(text) = std::fs::read_to_string(&toml) {
                let cases = parse(&text).map_err(|e| format!("{}: {e}", toml.display()));
                let cases = cases.unwrap();
                assert!(!cases.is_empty(), "{} lists no case", toml.display());
                if let Err(e) = sources(&package, &cases) {
                    panic!("{}: {e}", toml.display());
                }
                read += 1;
            }
        }
        assert!(read > 100, "only {read} tests.toml files");
    }
}
