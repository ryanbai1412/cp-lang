//! Something that can be run on an input: a solution, a brute, a
//! generator, a checker.
//!
//! A path becomes a program by its extension: `.cpp` is compiled with
//! `g++ -std=c++17 -O2` (once; the binary is cached by content), `.py`
//! runs under `python3`, and anything else is run as it is. A `.cp` file
//! goes through the compiler's C++ back end first, the same one behind
//! `cpl build`.

use cpl_diag::Shown;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How a run ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Exit {
    Code(i32),
    /// Killed by a signal: a crash.
    Signal,
    TimedOut,
}

/// What a run printed and what it cost.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub exit: Exit,
    pub elapsed: Duration,
    /// Peak resident memory in kilobytes, sampled while the program ran.
    /// `None` where the system does not report it.
    pub peak_kb: Option<u64>,
}

impl Outcome {
    pub fn ok(&self) -> bool {
        self.exit == Exit::Code(0)
    }
}

/// A command line.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    /// What the user called it: the path they gave.
    pub name: String,
    pub argv: Vec<String>,
    /// A `cpl judge` run, which takes `--diagnostic <file>` to write
    /// where it trapped there as JSON.
    pub says_trap: bool,
}

/// Why a path could not be made into a program.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BuildError(pub String);

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A stable hash (FNV-1a), for cache names.
fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Where built binaries are kept.
fn cache_dir() -> PathBuf {
    match cpl_env::CACHE.path() {
        Some(dir) => dir,
        None => std::env::temp_dir().join("cpl-cache"),
    }
}

/// The flags every C++ file is built with. The precompiled header is
/// only usable by a build with the same ones.
const CXX_FLAGS: [&str; 2] = ["-std=c++17", "-O2"];

/// Windows has no `ulimit`: a program's stack is the one it was linked
/// with, so C++ is linked with the 256 MB stack Windows judges give.
const WINDOWS_STACK: &str = "-Wl,--stack,268435456";

/// A header that includes the whole standard library, with its
/// precompiled form beside it, or `None` while there is none.
///
/// Most of the wait for a small C++ file is g++ reading the standard
/// headers. Read once and kept, they cost a fraction of that. The header
/// belongs to the compiler, not to a program, so it is kept per g++
/// version in the temp dir rather than in the binary cache. The first
/// run that finds it missing starts building it and goes on without it.
/// `CPL_NO_PCH` turns it off.
fn std_header() -> Option<&'static Path> {
    static HEADER: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    HEADER.get_or_init(find_std_header).as_deref()
}

#[cfg(not(unix))]
fn find_std_header() -> Option<PathBuf> {
    None
}

#[cfg(unix)]
fn find_std_header() -> Option<PathBuf> {
    if cpl_env::NO_PCH.is_set() {
        return None;
    }
    let version = Command::new("g++")
        .args(["-dumpfullversion", "-dumpmachine"])
        .output()
        .ok()?;
    let mut key = version.stdout;
    key.extend(CXX_FLAGS.concat().bytes());
    let dir = std::env::temp_dir()
        .join("cpl-pch")
        .join(format!("{:016x}", hash(&key)));
    let header = dir.join("cpl_std.h");
    let compiled = dir.join("cpl_std.h.gch");
    if compiled.is_file() {
        return Some(header);
    }
    std::fs::create_dir_all(&dir).ok()?;
    // One build at a time: a marker left by a build that died is
    // ignored once it is older than any build takes.
    let marker = dir.join("building");
    let stale = |path: &Path| {
        let age = path
            .metadata()
            .and_then(|m| m.modified())
            .ok()?
            .elapsed()
            .ok()?;
        Some(age > Duration::from_secs(120))
    };
    if marker.exists() && stale(&marker) != Some(true) {
        return None;
    }
    std::fs::write(&marker, b"").ok()?;
    std::fs::write(&header, "#include <bits/stdc++.h>\n").ok()?;
    // Left running after this process ends; the rename makes the
    // header appear whole or not at all.
    let script = r#"g++ "$@" -x c++-header "$HDR" -o "$HDR.gch.$$" && mv "$HDR.gch.$$" "$HDR.gch"; rm -f "$MARK""#;
    Command::new("sh")
        .args(["-c", script, "sh"])
        .args(CXX_FLAGS)
        .env("HDR", &header)
        .env("MARK", &marker)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    None
}

/// Whether the first thing `text` does is include the whole standard
/// library (`bits/stdc++.h`, or `bits/extc++.h`, which starts with it): then including it beforehand changes nothing. A file that
/// first defines a macro or sets a pragma may mean it for the headers.
fn starts_with_std(text: &[u8]) -> bool {
    let text = String::from_utf8_lossy(text);
    let mut in_comment = false;
    for line in text.lines() {
        let line = line.trim();
        if in_comment {
            in_comment = !line.contains("*/");
            continue;
        }
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.starts_with("/*") {
            in_comment = !line.contains("*/");
            continue;
        }
        let rest = line.strip_prefix('#').map(str::trim_start);
        let included = rest.and_then(|r| r.strip_prefix("include")).map(str::trim);
        // `bits/extc++.h` includes `bits/stdc++.h` first.
        return included.is_some_and(|name| {
            let name = name.trim_start_matches(['<', '"']);
            name.starts_with("bits/stdc++.h") || name.starts_with("bits/extc++.h")
        });
    }
    false
}

/// Builds `source`. `ours` is for C++ the compiler emitted: it includes
/// standard headers only, and no macro of its own comes before them.
fn compile_cpp(source: &Path, ours: bool) -> Result<String, BuildError> {
    let flags = CXX_FLAGS.map(String::from);
    compile_with(source, ours, "g++", &flags, None)
}

/// The judge's grader and the headers beside it, which it includes: what
/// a build linked against it depends on.
fn grader_bytes(grader: &Path) -> Result<Vec<u8>, BuildError> {
    let read = |path: &Path| {
        std::fs::read(path).map_err(|e| BuildError(format!("can't read {}: {e}", path.shown())))
    };
    let mut bytes = read(grader)?;
    let dir = grader
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut headers: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    headers.retain(|p| p.extension().is_some_and(|e| e == "h" || e == "hpp"));
    headers.sort();
    for header in headers {
        bytes.push(0);
        bytes.extend(read(&header)?);
    }
    Ok(bytes)
}

/// The C++ declarations of the fns that cross to the judge's grader, as a
/// header that stands alone, from a program's C++ (R10.1.3).
fn grader_boundary(cpp: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(cpp);
    let (open, close) = cpl_ir::GRADER_BOUNDARY;
    let (_, rest) = text.split_once(open)?;
    let (decls, _) = rest.split_once(close)?;
    Some(format!(
        "#include <string>\n#include <tuple>\n#include <utility>\n#include <vector>\n{decls}"
    ))
}

fn compile_with(
    source: &Path,
    ours: bool,
    compiler: &str,
    flags: &[String],
    grader: Option<&Path>,
) -> Result<String, BuildError> {
    let mut text = std::fs::read(source)
        .map_err(|e| BuildError(format!("can't read {}: {e}", source.shown())))?;
    let usual = compiler == "g++" && flags.iter().eq(CXX_FLAGS.iter());
    let source_len = text.len();
    if let Some(grader) = grader {
        text.extend(b"\0grader\0");
        text.extend(grader_bytes(grader)?);
    }
    if !usual {
        text.extend(format!("\0{compiler} {}", flags.join(" ")).bytes());
    }
    if cfg!(windows) {
        text.extend(format!("\0{WINDOWS_STACK}").bytes());
    }
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| BuildError(format!("can't create {}: {e}", dir.shown())))?;
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let binary = dir.join(format!("{stem}-{:016x}", hash(&text)));
    if !binary.is_file() {
        // Built under another name, then renamed, so a build that is
        // interrupted or raced never leaves half a binary behind.
        static BUILDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let partial = dir.join(format!(
            "{stem}-{:016x}.{}.{}.tmp",
            hash(&text),
            std::process::id(),
            BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let mut build = Command::new(compiler);
        build.args(flags);
        if cfg!(windows) {
            build.arg(WINDOWS_STACK);
        }
        // The grader is a second file the header would be forced into.
        if usual && grader.is_none() && (ours || starts_with_std(&text[..source_len])) {
            if let Some(header) = std_header() {
                build.arg("-include").arg(header);
            }
        }
        if let Some(grader) = grader {
            let header = dir.join(format!("{stem}-{:016x}.grader.h", hash(&text)));
            if let Some(boundary) = grader_boundary(&text[..source_len]) {
                std::fs::write(&header, boundary)
                    .map_err(|e| BuildError(format!("can't write {}: {e}", header.shown())))?;
                // Both files see the program's declarations, so a grader
                // declaring a fn otherwise ("ambiguating new declaration")
                // fails to compile instead of linking and crashing.
                build.arg("-include").arg(&header);
            } else if ours {
                return Err(BuildError(format!(
                    "{} has no fn the judge's grader {} supplies or calls",
                    source.shown(),
                    grader.shown()
                )));
            }
        }
        build.arg("-o").arg(&partial).arg(source);
        if let Some(grader) = grader {
            let dir = grader.parent().filter(|d| !d.as_os_str().is_empty());
            build
                .arg("-I")
                .arg(dir.unwrap_or(Path::new(".")))
                .arg(grader);
        }
        let output = build.output().map_err(|e| {
            BuildError(format!(
                "can't run {compiler} to build {}: {e}",
                source.shown()
            ))
        })?;
        if !output.status.success() {
            let errors = String::from_utf8_lossy(&output.stderr);
            let shown: Vec<&str> = errors.lines().take(20).collect();
            let what = match grader {
                Some(grader) => format!(
                    "{} with the judge's grader {}",
                    source.shown(),
                    grader.shown()
                ),
                None => source.shown().to_string(),
            };
            return Err(BuildError(format!(
                "{what} does not compile:\n{}",
                shown.join("\n")
            )));
        }
        // A raced build may have renamed the same bytes into place first,
        // and Windows refuses to replace a binary that is running.
        if let Err(e) = std::fs::rename(&partial, &binary) {
            let _ = std::fs::remove_file(&partial);
            if !binary.is_file() {
                return Err(BuildError(format!("can't write {}: {e}", binary.shown())));
            }
        }
    }
    Ok(binary.display().to_string())
}

/// Which program of a `.cp` file to build.
#[derive(Clone, Copy, Debug)]
enum Part {
    Solution,
    ColdCache,
    Brute,
}

/// A counterexample that crashes, or overflows, is one stress must see.
const LOCAL: cpl_codegen::Checks = cpl_codegen::Checks::LOCAL;

/// Emits the C++ for a `.cp` file into the cache, through the same front
/// end and back end as `cpl build`. Returns the path of the C++ file.
fn emit_cpp(
    source: &Path,
    text: Option<&str>,
    part: Part,
    checks: cpl_codegen::Checks,
) -> Result<(PathBuf, cpl_judge::Profile), BuildError> {
    let name = source.shown();
    let text = match text {
        Some(text) => text.to_string(),
        None => std::fs::read_to_string(source)
            .map_err(|e| BuildError(format!("can't read {name}: {e}")))?,
    };
    let profile = cpl_judge::resolve(source, &text, None).map_err(BuildError)?;
    let key = emitted_key(&name, &text, part, checks, &profile);
    if let Some(cpp) = emitted(&key) {
        return Ok((cpp, profile));
    }
    let errors = |diagnostics: &[cpl_diag::Diagnostic]| {
        BuildError(format!(
            "{name} does not build:\n{}",
            cpl_diag::render_all(diagnostics, &name, &text).trim_end()
        ))
    };
    let mut db = cpl_check::Session::new();
    let analysis = db.analyze_text(&name, text.as_str());
    let is_error = |d: &cpl_diag::Diagnostic| d.severity == cpl_diag::Severity::Error;
    if analysis.diagnostics.iter().any(is_error) {
        return Err(errors(&analysis.diagnostics));
    }
    let missing = |kind: &str| BuildError(format!("{name} declares no `{kind}` fn"));
    let lowered = match part {
        Part::Solution => cpl_ir::lower_sampling_laws(&analysis, &db.facts_source()),
        Part::ColdCache => cpl_ir::lower_cold_cache(&analysis, &db.facts_source()),
        Part::Brute => {
            let (brute_fn, _) = cpl_ir::brute(&analysis).ok_or_else(|| missing("@alt(program)"))?;
            cpl_ir::lower_brute(&analysis, &db.facts_source(), &brute_fn)
        }
    };
    let program = lowered.map_err(|diagnostics| errors(&diagnostics))?;
    let file = source.file_name().unwrap_or_default().to_string_lossy();
    let target = cpl_codegen::Target {
        stamp: Some(profile.stamp()),
        thread_stack: profile.thread_stack,
        windows: profile.os == cpl_judge::Os::Windows,
        checks,
        cpu: profile.cpu.clone(),
    };
    let cpp = cpl_codegen::emit_for(&program, &file, None, &target);
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| BuildError(format!("can't create {}: {e}", dir.shown())))?;
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let out = dir.join(format!("{stem}-{:016x}.cpp", hash(cpp.as_bytes())));
    std::fs::write(&out, cpp)
        .map_err(|e| BuildError(format!("can't write {}: {e}", out.shown())))?;
    remember_emitted(&key, &out, &analysis);
    Ok((out, profile))
}

/// What the C++ of a `.cp` file depends on beyond the files it reads:
/// this `cpl` (and the std built into it), the part, its checks and its judge.
fn emitted_key(
    name: &str,
    text: &str,
    part: Part,
    checks: cpl_codegen::Checks,
    profile: &cpl_judge::Profile,
) -> String {
    let exe = std::env::current_exe()
        .and_then(|exe| exe.metadata())
        .map(|m| format!("{} {:?}", m.len(), m.modified().ok()))
        .unwrap_or_default();
    let mut key = format!("{exe}\0{part:?}\0{checks:?}\0{profile:?}\0{name}\0");
    key.push_str(text);
    let stem = Path::new(name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    format!("{stem}-{:016x}.emitted", hash(key.as_bytes()))
}

/// The C++ an earlier build emitted under `key`, while every file that
/// build read still has the bytes it read. A rerun of `cpl test` or
/// `cpl stress` on an unchanged file skips the front end.
fn emitted(key: &str) -> Option<PathBuf> {
    let record = std::fs::read_to_string(cache_dir().join(key)).ok()?;
    let mut lines = record.lines();
    let cpp = PathBuf::from(lines.next()?);
    for line in lines {
        let (bytes, path) = line.split_once('\t')?;
        if format!("{:016x}", hash(&std::fs::read(path).ok()?)) != bytes {
            return None;
        }
    }
    cpp.is_file().then_some(cpp)
}

/// Records that `cpp` is what `key` emits, with each file `analysis` read.
/// The bundled std is part of `cpl` itself, which the key names.
fn remember_emitted(key: &str, cpp: &Path, analysis: &cpl_check::Analysis) {
    let mut read = std::collections::BTreeSet::new();
    let mut pending: Vec<&cpl_check::Analysis> = vec![analysis];
    while let Some(next) = pending.pop() {
        for module in next.modules() {
            let bundled = !cpl_check::std_from_disk()
                && (module.name == "std" || module.name.starts_with("std."));
            if !bundled && read.insert(module.path.as_str()) {
                pending.push(&module.analysis);
            }
        }
    }
    let mut record = format!("{}\n", cpp.display());
    for path in read {
        let Ok(bytes) = std::fs::read(path) else {
            // A module that is no file (an editor buffer) can't be checked again.
            return;
        };
        record.push_str(&format!("{:016x}\t{path}\n", hash(&bytes)));
    }
    let file = cache_dir().join(key);
    let partial = file.with_extension(format!("{}.tmp", std::process::id()));
    if std::fs::write(&partial, record).is_ok() && std::fs::rename(&partial, &file).is_err() {
        let _ = std::fs::remove_file(&partial);
    }
}

/// Builds the C++ of a `.cp` file as its judge does, as far as the
/// compiler here allows, and runs it on the judge's stack.
fn build_judged(
    source: &Path,
    text: Option<&str>,
    part: Part,
    checks: cpl_codegen::Checks,
) -> Result<Vec<String>, BuildError> {
    build_linked(source, text, part, checks, None)
}

/// [`build_judged`], linked against the judge's `grader` when there is one.
fn build_linked(
    source: &Path,
    text: Option<&str>,
    part: Part,
    checks: cpl_codegen::Checks,
    grader: Option<&Path>,
) -> Result<Vec<String>, BuildError> {
    let (cpp, profile) = emit_cpp(source, text, part, checks)?;
    let host = profile.host();
    if !found(&host.compiler) {
        if let Some(grader) = grader {
            return Err(BuildError(format!(
                "{} runs under the judge's grader {}, which needs a C++ compiler, and there is no {} here",
                source.shown(),
                grader.shown(),
                host.compiler
            )));
        }
        return interpreted(source, text, part);
    }
    if grader.is_none() {
        let text = std::fs::read(&cpp).unwrap_or_default();
        if let Some(boundary) = grader_boundary(&text) {
            let crossing: Vec<&str> = boundary.lines().filter(|l| !l.starts_with('#')).collect();
            return Err(BuildError(format!(
                "{} links only with the judge's grader, which supplies or calls `{}`: \
                 pass `--grader grader.cpp`, or put grader.cpp beside its tests",
                source.shown(),
                crossing.join(" ").trim_end_matches(';')
            )));
        }
    }
    let binary = compile_with(&cpp, true, &host.compiler, &host.flags, grader)?;
    let stack = profile.ulimit();
    // Off Unix there is no `ulimit`: the stack is the one it was linked with.
    if cfg!(not(unix)) || current_stack().as_deref() == Some(stack.as_str()) {
        return Ok(vec![binary]);
    }
    Ok(profile.on_stack(binary))
}

/// The C++ compiler `source`'s judge builds with here, when this machine
/// has none by that name, so its release build runs in the interpreter.
pub fn missing_compiler(source: &Path) -> Option<String> {
    let text = std::fs::read_to_string(source).ok()?;
    let host = cpl_judge::resolve(source, &text, None).ok()?.host();
    (!found(&host.compiler)).then_some(host.compiler)
}

/// Whether `compiler` can be started here: a file, or a name on the
/// PATH. Looked up, not started, so a cached build costs no process.
fn found(compiler: &str) -> bool {
    type Asked = std::sync::Mutex<std::collections::HashMap<String, bool>>;
    static FOUND: std::sync::OnceLock<Asked> = std::sync::OnceLock::new();
    let mut found = FOUND
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *found
        .entry(compiler.to_string())
        .or_insert_with(|| on_path(compiler).is_some())
}

/// `part` of `source` run by `cpl run` in the interpreter, on a machine
/// with no C++ compiler: slower, but the same answers.
fn interpreted(source: &Path, text: Option<&str>, part: Part) -> Result<Vec<String>, BuildError> {
    let cpl = std::env::current_exe().map_err(|e| {
        BuildError(format!(
            "can't find `cpl` to interpret {}: {e}",
            source.shown()
        ))
    })?;
    if let Some(text) = text {
        std::fs::write(source, text)
            .map_err(|e| BuildError(format!("can't write {}: {e}", source.shown())))?;
    }
    let mut argv = vec![cpl.display().to_string(), "run".to_string()];
    match part {
        Part::Solution | Part::ColdCache => {}
        Part::Brute => argv.push("--brute".to_string()),
    }
    argv.push(source.display().to_string());
    Ok(argv)
}

/// This process's `ulimit -s`, then its hard cap (`ulimit -Hs`); `None` off Unix.
fn stacks() -> Option<&'static (String, String)> {
    static STACKS: std::sync::OnceLock<Option<(String, String)>> = std::sync::OnceLock::new();
    STACKS
        .get_or_init(|| {
            if cfg!(not(unix)) {
                return None;
            }
            // Linux says it without a shell.
            if let Ok(limits) = std::fs::read_to_string("/proc/self/limits") {
                let line = limits
                    .lines()
                    .find_map(|line| line.strip_prefix("Max stack size"))?;
                let kb = |limit: &str| match limit.parse::<u64>() {
                    Ok(bytes) => (bytes >> 10).to_string(),
                    Err(_) => limit.to_string(),
                };
                let mut limits = line.split_whitespace().map(kb);
                return Some((limits.next()?, limits.next()?));
            }
            let out = Command::new("sh")
                .args(["-c", "ulimit -s; ulimit -Hs"])
                .output()
                .ok()?;
            let text = String::from_utf8_lossy(&out.stdout).into_owned();
            let mut lines = text.lines().map(|line| line.trim().to_string());
            Some((lines.next()?, lines.next()?))
        })
        .as_ref()
}

const SHELL: &str = "/bin/sh";

/// The `ulimit -s` programs started from here get: the hard cap.
fn current_stack() -> Option<String> {
    stacks().map(|(_, hard)| hard.clone())
}

/// The stack [`Program::to_command`] raises a program's to, when this
/// process's own is below the hard cap.
fn raised_stack() -> Option<&'static str> {
    stacks()
        .filter(|(soft, hard)| soft != hard && Path::new(SHELL).is_file())
        .map(|(_, hard)| hard.as_str())
}

/// The `setpriv` that makes a program get SIGKILL when the thread that
/// started it ends; `None` where there is none, as on macOS and Windows.
fn dies_with_parent() -> Option<&'static str> {
    static SETPRIV: std::sync::OnceLock<Option<&'static str>> = std::sync::OnceLock::new();
    *SETPRIV.get_or_init(|| {
        if cfg!(not(target_os = "linux")) || !Path::new(SHELL).is_file() {
            return None;
        }
        ["/usr/bin/setpriv", "/bin/setpriv"]
            .into_iter()
            .find(|setpriv| {
                Command::new(setpriv)
                    .args(["--pdeathsig", "KILL", "--", "true"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .is_ok_and(|status| status.success())
            })
    })
}

/// The interpreter and its arguments a script's first line names:
/// `#!/bin/sh` is `["/bin/sh"]`, `#!/usr/bin/env python3 -u` is
/// `["python3", "-u"]`. `None` without a `#!`.
pub fn interpreter(first_line: &str) -> Option<Vec<String>> {
    let words: Vec<String> = first_line
        .strip_prefix("#!")?
        .split_whitespace()
        .map(String::from)
        .collect();
    let (program, rest) = words.split_first()?;
    if interpreter_name(program) == "env" {
        let rest: Vec<String> = rest.iter().filter(|w| *w != "-S").cloned().collect();
        return (!rest.is_empty()).then_some(rest);
    }
    Some(words)
}

/// How to run Python 3: `python3`, or on Windows the `py -3` launcher, else
/// `python`. Windows' `python3` is often the Store's stub, which never runs
/// the script.
pub fn python() -> Vec<String> {
    if cfg!(windows) {
        let real = |name: &str| find_interpreter(name).filter(|p| !p.contains("WindowsApps"));
        if real("py").is_some() {
            return vec!["py".to_string(), "-3".to_string()];
        }
        if real("python").is_some() {
            return vec!["python".to_string()];
        }
    }
    vec!["python3".to_string()]
}

/// `/usr/bin/python3` is `python3`.
fn interpreter_name(program: &str) -> &str {
    program.rsplit(['/', '\\']).next().unwrap_or(program)
}

/// Where `program` from a `#!` line runs: the path itself when it exists
/// (`/bin/sh` on Linux and macOS), else its name on the PATH (Git for
/// Windows' `sh.exe`).
fn find_interpreter(program: &str) -> Option<String> {
    if Path::new(program).is_file() {
        return Some(program.to_string());
    }
    on_path(interpreter_name(program))
}

/// `name` as a file, or the first file of that name on the PATH (with
/// Windows' `.exe` and `.cmd`).
fn on_path(name: &str) -> Option<String> {
    if name.contains(['/', '\\']) {
        return Path::new(name).is_file().then(|| name.to_string());
    }
    let exts: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ""]
    } else {
        &[""]
    };
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|dir| {
        exts.iter()
            .map(|ext| dir.join(format!("{name}{ext}")))
            .find(|path| path.is_file())
            .map(|path| path.display().to_string())
    })
}

impl Program {
    /// A program that runs `argv` as given.
    pub fn command(name: &str, argv: &[&str]) -> Program {
        Program {
            name: name.to_string(),
            argv: argv.iter().map(|arg| arg.to_string()).collect(),
            says_trap: false,
        }
    }

    /// Makes `path` runnable, building it if it is source. A path that is
    /// not a file but has spaces in it is a command line, split like a
    /// shell splits one: `--gen "python3 gen.py --tree"`.
    pub fn resolve(path: &Path) -> Result<Program, BuildError> {
        let name = path.display().to_string();
        if !path.is_file() && name.contains(char::is_whitespace) {
            let words = split_command(&name)?;
            let argv: Vec<&str> = words.iter().map(String::as_str).collect();
            return Ok(Program::command(&name, &argv));
        }
        if !path.is_file() {
            return Err(BuildError(format!("there is no file {name}")));
        }
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let argv = match extension {
            "cpp" | "cc" | "cxx" => vec![compile_cpp(path, false)?],
            "py" => {
                let mut argv = python();
                argv.push(name.clone());
                argv
            }
            "cp" => build_judged(path, None, Part::Solution, LOCAL)?,
            _ => {
                // Relative paths need `./` to be found.
                let canonical = path
                    .canonicalize()
                    .map_err(|e| BuildError(format!("can't find {name}: {e}")))?;
                let full = canonical.display().to_string();
                // A script runs through the interpreter its `#!` line names on
                // every OS, so Windows (which ignores `#!`) and a script that
                // isn't executable behave as Linux does.
                let first = std::fs::read(path)
                    .map(|bytes| bytes.split(|&b| b == b'\n').next().unwrap_or(&[]).to_vec())
                    .unwrap_or_default();
                match interpreter(&String::from_utf8_lossy(&first)) {
                    Some(mut argv) => {
                        argv[0] = find_interpreter(&argv[0]).ok_or_else(|| {
                            BuildError(format!(
                                "{name} starts `{}`, and there is no `{}` on the PATH{}",
                                String::from_utf8_lossy(&first).trim_end(),
                                interpreter_name(&argv[0]),
                                if cfg!(windows) {
                                    " (Git for Windows has `sh` and `bash`)"
                                } else {
                                    ""
                                }
                            ))
                        })?;
                        // `D:/a/gen.sh`: Git for Windows' `sh` can't open `\\?\D:\a\gen.sh`.
                        argv.push(cpl_diag::shown_path(&canonical));
                        argv
                    }
                    None => vec![full],
                }
            }
        };
        Ok(Program {
            name,
            argv,
            says_trap: false,
        })
    }

    /// The `.cp` file at `path` built for its judge with `checks` instead
    /// of the traps stress keeps (`cpl run --release --checks`).
    pub fn resolve_checks(path: &Path, checks: cpl_codegen::Checks) -> Result<Program, BuildError> {
        Ok(Program {
            name: path.shown(),
            argv: build_judged(path, None, Part::Solution, checks)?,
            says_trap: false,
        })
    }

    /// The `.cp` file at `path` linked against the judge's `grader` (a
    /// `.cpp` with the `main`, beside the headers it includes): how a
    /// program of `@grader` fns runs here as it does on the judge.
    pub fn resolve_with_grader(path: &Path, grader: &Path) -> Result<Program, BuildError> {
        if !grader.is_file() {
            return Err(BuildError(format!("there is no grader {}", grader.shown())));
        }
        Ok(Program {
            name: path.shown(),
            argv: build_linked(path, None, Part::Solution, LOCAL, Some(grader))?,
            says_trap: false,
        })
    }

    /// The `.cp` source `text`, built as if it were the file `path`, which
    /// need not exist: its imports and judge resolve from beside `path`.
    pub fn resolve_source(path: &Path, text: &str) -> Result<Program, BuildError> {
        Ok(Program {
            name: path.shown(),
            argv: build_judged(path, Some(text), Part::Solution, LOCAL)?,
            says_trap: false,
        })
    }

    /// The solution in `@cache` cold mode (cache writes through read `self`
    /// are undone at method exit), to check P1 against the normal run.
    pub fn resolve_cold_cache(path: &Path) -> Result<Program, BuildError> {
        Ok(Program {
            name: format!("{} (@cache cold)", path.shown()),
            argv: build_judged(path, None, Part::ColdCache, LOCAL)?,
            says_trap: false,
        })
    }

    /// The `@alt(program)` fn of the `.cp` file `path`, built like the
    /// solution: the file's input header, then the fn.
    pub fn resolve_brute(path: &Path, fn_name: &str) -> Result<Program, BuildError> {
        Ok(Program {
            name: format!("{} (`{fn_name}`)", path.shown()),
            argv: build_judged(path, None, Part::Brute, LOCAL)?,
            says_trap: false,
        })
    }

    /// The file's `@gen` fn lowered for the VM, which draws every input.
    pub fn lowered_gen(path: &Path, size: i64) -> Result<cpl_ir::Program, BuildError> {
        let name = path.shown();
        let text = std::fs::read_to_string(path)
            .map_err(|e| BuildError(format!("can't read {name}: {e}")))?;
        let mut db = cpl_check::Session::new();
        let analysis = db.analyze_text(&name, text.as_str());
        let failed = |diagnostics: &[cpl_diag::Diagnostic]| {
            BuildError(format!(
                "{name} does not build:\n{}",
                cpl_diag::render_all(diagnostics, &name, &text).trim_end()
            ))
        };
        let is_error = |d: &cpl_diag::Diagnostic| d.severity == cpl_diag::Severity::Error;
        if analysis.diagnostics.iter().any(is_error) {
            return Err(failed(&analysis.diagnostics));
        }
        let (gen_fn, _) = cpl_ir::generator(&analysis)
            .ok_or_else(|| BuildError(format!("{name} declares no `@gen` fn")))?;
        cpl_ir::lower_gen(&analysis, &db.facts_source(), &gen_fn, size)
            .map_err(|diagnostics| failed(&diagnostics))
    }

    /// The command with `extra` arguments added.
    pub fn with_args(&self, extra: &[&str]) -> Program {
        let mut program = self.clone();
        program.argv.extend(extra.iter().map(|arg| arg.to_string()));
        program
    }

    /// The one way a program is started from here: the solution, brute,
    /// generator, checker and interactor alike. It gets the stack judges
    /// give, the hard cap (`unlimited` where allowed), not this shell's
    /// 8 MB, so an accepted C++ keeping big arrays on the stack runs.
    /// On Linux it also dies with this process: in its own process group
    /// it misses the Ctrl-C or kill that ends `cpl`, and would run on.
    pub(crate) fn to_command(&self) -> Command {
        let stack = raised_stack();
        let orphan_kill = dies_with_parent();
        let command = match (stack, orphan_kill) {
            (None, None) => {
                let mut command = Command::new(&self.argv[0]);
                command.args(&self.argv[1..]);
                command
            }
            _ => {
                // By its path: a program may run with no `PATH` at all.
                let mut command = Command::new(SHELL);
                let raise = match stack {
                    Some(_) => "ulimit -s \"$0\" 2>/dev/null; ",
                    None => "",
                };
                let exec = match orphan_kill {
                    Some(setpriv) => format!("exec {setpriv} --pdeathsig KILL -- \"$@\""),
                    None => "exec \"$@\"".to_string(),
                };
                command
                    .arg("-c")
                    .arg(format!("{raise}{exec}"))
                    .arg(stack.unwrap_or("sh"))
                    .args(&self.argv);
                command
            }
        };
        // Its own process group, so a time limit stops what it started too.
        #[cfg(unix)]
        let command = {
            let mut command = command;
            std::os::unix::process::CommandExt::process_group(&mut command, 0);
            command
        };
        command
    }

    /// Runs the program on `input` for at most `limit`.
    pub fn run(&self, input: &str, limit: Duration) -> std::io::Result<Outcome> {
        let start = Instant::now();
        let mut child = self
            .to_command()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = piped(child.stdin.take())?;
        let mut stdout = piped(child.stdout.take())?;
        let mut stderr = piped(child.stderr.take())?;
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let (exit, peak_kb, elapsed) = std::thread::scope(|scope| {
            // Each pipe has its own thread, so a program that prints a
            // lot or never reads its input cannot block either side.
            scope.spawn(move || {
                let _ = stdin.write_all(input.as_bytes());
            });
            scope.spawn(|| {
                let _ = stdout.read_to_end(&mut out);
            });
            scope.spawn(|| {
                let _ = stderr.read_to_end(&mut err);
            });
            supervise(child, start, limit)
        });
        Ok(Outcome {
            // LF everywhere: a Windows program's `\r\n` never reaches a compare, report or saved case.
            stdout: String::from_utf8_lossy(&out).replace("\r\n", "\n"),
            stderr: String::from_utf8_lossy(&err).replace("\r\n", "\n"),
            exit,
            elapsed,
            peak_kb,
        })
    }

    /// Runs the program on this process's own stdin for at most `limit`,
    /// passing what it prints to `out` as it prints it, so it can talk to
    /// whatever is on the other end (a judge, a person). The outcome's
    /// stdout is empty: it went to `out`.
    pub fn run_live(
        &self,
        out: &mut (dyn Write + Send),
        limit: Duration,
    ) -> std::io::Result<Outcome> {
        let start = Instant::now();
        let mut child = self
            .to_command()
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdout = piped(child.stdout.take())?;
        let mut stderr = piped(child.stderr.take())?;
        let mut err = Vec::new();
        let (exit, peak_kb, elapsed) = std::thread::scope(|scope| {
            scope.spawn(move || {
                let mut chunk = vec![0; 1 << 16];
                while let Ok(read @ 1..) = stdout.read(&mut chunk) {
                    if out
                        .write_all(&chunk[..read])
                        .and_then(|()| out.flush())
                        .is_err()
                    {
                        break;
                    }
                }
            });
            scope.spawn(|| {
                let _ = stderr.read_to_end(&mut err);
            });
            supervise(child, start, limit)
        });
        Ok(Outcome {
            stdout: String::new(),
            stderr: String::from_utf8_lossy(&err).replace("\r\n", "\n"),
            exit,
            elapsed,
            peak_kb,
        })
    }
}

/// Waits for `child` to end, stopping it once it is charged more than
/// `limit`, and samples its peak memory meanwhile; its exit wakes the wait
/// at once. It is charged its CPU time, as judges charge it, so a loaded
/// machine doesn't make it late; one that idles (sleeps, or waits on
/// something that never comes) stops once it has slept [`idle_limit`],
/// and time it spends ready to run but waiting for a CPU is not sleep.
fn supervise(
    child: std::process::Child,
    start: Instant,
    limit: Duration,
) -> (Exit, Option<u64>, Duration) {
    let pid = child.id();
    let mut child = cpl_wait::on_exit(child);
    let mut charge = Charge::new(pid);
    let mut peak = None;
    let idle = idle_limit(limit);
    loop {
        peak = peak_memory(pid).max(peak);
        charge.sample();
        let spent = charge.spent(|| start.elapsed());
        let tick = Instant::now() + Duration::from_micros(300);
        match child.by(tick) {
            Ok(Some(_)) if spent > limit => return (Exit::TimedOut, peak, spent),
            Ok(Some(status)) => {
                let exit = status.code().map_or(Exit::Signal, Exit::Code);
                return (exit, peak, spent);
            }
            Ok(None) if spent > limit => {
                let _ = child.kill();
                return (Exit::TimedOut, peak, spent);
            }
            Ok(None) if charge.slept(|| start.elapsed()) >= idle => {
                let _ = child.kill();
                return (Exit::TimedOut, peak, start.elapsed());
            }
            Ok(None) => {}
            Err(_) => return (Exit::Signal, peak, spent),
        }
    }
}

/// How long a run may sleep while charged less than `limit`: a program
/// this far behind is idle, not slow.
fn idle_limit(limit: Duration) -> Duration {
    limit * 3
}

/// What a program is charged for a run: its own CPU time, user and system,
/// as judges charge it, sampled until it is reaped; where the system
/// doesn't say, the caller's fallback clock. Every native run, piped or
/// interactive, is charged through this.
pub(crate) struct Charge {
    pid: u32,
    cpu: Option<Duration>,
    slept: Option<Duration>,
    sampled: Instant,
}

impl Charge {
    pub(crate) fn new(pid: u32) -> Self {
        Charge {
            pid,
            cpu: None,
            slept: None,
            sampled: Instant::now(),
        }
    }

    /// Notes the program's CPU time so far, and the time since the last
    /// note as sleep when it is asleep now; the last note before it is
    /// reaped is what it is charged.
    pub(crate) fn sample(&mut self) {
        let now = Instant::now();
        if let Some((cpu, asleep)) = cpu_time(self.pid) {
            self.cpu = Some(cpu);
            let slept = self.slept.get_or_insert(Duration::ZERO);
            if asleep {
                *slept += now - self.sampled;
            }
        }
        self.sampled = now;
    }

    /// How long it has slept, waiting on input or a timer: not the time a
    /// loaded machine kept it waiting for a CPU. `fallback` where the
    /// system doesn't say.
    pub(crate) fn slept(&self, fallback: impl FnOnce() -> Duration) -> Duration {
        self.slept.unwrap_or_else(fallback)
    }

    /// The time charged so far: the CPU time, or `fallback` where there is none.
    pub(crate) fn spent(&self, fallback: impl FnOnce() -> Duration) -> Duration {
        self.cpu.unwrap_or_else(fallback)
    }
}

/// The words of a command line: split at spaces, with '...' and "..."
/// keeping spaces and a backslash escaping the next character.
fn split_command(line: &str) -> Result<Vec<String>, BuildError> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quote: Option<char> = None;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some('"') | None, '\\') => match chars.next() {
                Some(next) => word.get_or_insert_with(String::new).push(next),
                None => return Err(BuildError(format!("`{line}` ends in a lone backslash"))),
            },
            (Some(_), c) => word.get_or_insert_with(String::new).push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                word.get_or_insert_with(String::new);
            }
            (None, c) if c.is_whitespace() => words.extend(word.take()),
            (None, c) => word.get_or_insert_with(String::new).push(c),
        }
    }
    if quote.is_some() {
        return Err(BuildError(format!(
            "`{line}` has a quote that is never closed"
        )));
    }
    words.extend(word);
    if words.is_empty() {
        return Err(BuildError("the command line is empty".into()));
    }
    Ok(words)
}

/// The high-water mark of a running process's memory, in kilobytes. None
/// until the child has exec'd: a forked child still shares our executable
/// and reports our own high-water mark, a false MLE when we are large.
fn peak_memory(pid: u32) -> Option<u64> {
    let exe = |pid: &str| std::fs::read_link(format!("/proc/{pid}/exe")).ok();
    if exe(&pid.to_string()).is_none_or(|child| exe("self") == Some(child)) {
        return None;
    }
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// The CPU time a running process has spent, user and system, with that
/// of the children it has waited for: what a judge charges it; and whether
/// it is asleep. `None` where the system doesn't say (no `/proc`) or once
/// it is gone.
fn cpu_time(pid: u32) -> Option<(Duration, bool)> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Fields after the parenthesized name: the state is the 1st; utime,
    // stime, cutime and cstime are the 12th to the 15th, in ticks of
    // 1/100 s (USER_HZ).
    let after = &stat[stat.rfind(')')? + 1..];
    let asleep = after.split_whitespace().next() == Some("S");
    let ticks: u64 = (after.split_whitespace().skip(11).take(4))
        .map(|field| field.parse::<u64>().ok())
        .sum::<Option<u64>>()?;
    Some((Duration::from_millis(ticks * 10), asleep))
}

#[cfg(test)]
mod tests {
    use super::{peak_memory, starts_with_std};

    #[cfg(unix)]
    #[test]
    fn a_program_s_output_reaches_stress_with_lf_line_ends() {
        let crlf = super::Program::command("crlf", &["printf", "1 2\\r\\n3\\r\\n"]);
        let ran = crlf.run("", std::time::Duration::from_secs(5)).unwrap();
        assert_eq!(ran.stdout, "1 2\n3\n");
    }

    #[test]
    fn a_child_that_has_not_exec_d_yet_has_no_peak_of_its_own() {
        assert_eq!(peak_memory(std::process::id()), None);
    }

    #[test]
    fn the_standard_header_is_only_put_first_where_it_already_is() {
        assert!(starts_with_std(
            b"// a brute\n\n#include <bits/stdc++.h>\nint main() {}\n"
        ));
        assert!(starts_with_std(
            b"/* two\n   lines */\n# include \"bits/extc++.h\"\n"
        ));
        // A macro or a pragma ahead of the headers may be meant for them.
        assert!(!starts_with_std(
            b"#define _GLIBCXX_DEBUG\n#include <bits/stdc++.h>\n"
        ));
        assert!(!starts_with_std(
            b"#pragma GCC optimize(\"O3\")\n#include <bits/stdc++.h>\n"
        ));
        assert!(!starts_with_std(
            b"#include <vector>\n#include <bits/stdc++.h>\n"
        ));
        assert!(!starts_with_std(b""));
    }
}

#[cfg(test)]
mod shebang_tests {
    use super::{interpreter, Program};

    #[test]
    fn a_scripts_first_line_names_its_interpreter() {
        let words = |line: &str| interpreter(line).map(|w| w.join(" "));
        assert_eq!(words("#!/bin/sh"), Some("/bin/sh".into()));
        assert_eq!(words("#! /bin/bash -e"), Some("/bin/bash -e".into()));
        assert_eq!(words("#!/usr/bin/env python3"), Some("python3".into()));
        assert_eq!(
            words("#!/usr/bin/env -S python3 -u\r"),
            Some("python3 -u".into())
        );
        assert_eq!(words("#!/usr/bin/env"), None);
        assert_eq!(words("echo 1"), None);
        assert_eq!(words("\u{7f}ELF"), None);
    }

    // The script is not executable: it runs through `sh`, as on Windows.
    #[cfg(unix)]
    #[test]
    fn a_script_runs_through_its_interpreter_without_the_execute_bit() {
        let dir = std::env::temp_dir().join(format!("cpl-shebang-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let script = dir.join("gen.sh");
        std::fs::write(&script, "#!/bin/sh\necho \"$1 $1\"\n").unwrap();
        let program = Program::resolve(&script).unwrap();
        assert_eq!(program.argv[0], "/bin/sh");
        let out = program
            .with_args(&["7"])
            .run("", std::time::Duration::from_secs(5))
            .unwrap();
        assert_eq!(out.stdout, "7 7\n");
        let missing = dir.join("odd.sh");
        std::fs::write(&missing, "#!/usr/bin/env cpl-no-such-shell\n").unwrap();
        let err = Program::resolve(&missing).unwrap_err().0;
        assert!(
            err.contains("there is no `cpl-no-such-shell` on the PATH"),
            "{err}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}

/// The pipe `spawn` was asked to make: an error rather than a panic if the
/// child has none.
pub fn piped<T>(pipe: Option<T>) -> std::io::Result<T> {
    pipe.ok_or_else(|| std::io::Error::other("the child process was started without its pipe"))
}
