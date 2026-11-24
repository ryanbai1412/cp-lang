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
    /// Variables set for it, on top of the ones `cpl` runs with.
    pub env: Vec<(String, String)>,
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
    match std::env::var_os("CPL_CACHE") {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir().join("cpl-cache"),
    }
}

/// The flags every C++ file is built with. The precompiled header is
/// only usable by a build with the same ones.
const CXX_FLAGS: [&str; 2] = ["-std=c++17", "-O2"];

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
    if std::env::var_os("CPL_NO_PCH").is_some() {
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
    compile_with(source, ours, "g++", &flags)
}

fn compile_with(
    source: &Path,
    ours: bool,
    compiler: &str,
    flags: &[String],
) -> Result<String, BuildError> {
    let mut text = std::fs::read(source)
        .map_err(|e| BuildError(format!("can't read {}: {e}", source.shown())))?;
    let usual = compiler == "g++" && flags.iter().eq(CXX_FLAGS.iter());
    let source_len = text.len();
    if !usual {
        text.extend(format!("\0{compiler} {}", flags.join(" ")).bytes());
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
        if usual && (ours || starts_with_std(&text[..source_len])) {
            if let Some(header) = std_header() {
                build.arg("-include").arg(header);
            }
        }
        let output = build
            .arg("-o")
            .arg(&partial)
            .arg(source)
            .output()
            .map_err(|e| {
                BuildError(format!(
                    "can't run {compiler} to build {}: {e}",
                    source.shown()
                ))
            })?;
        if !output.status.success() {
            let errors = String::from_utf8_lossy(&output.stderr);
            let shown: Vec<&str> = errors.lines().take(20).collect();
            return Err(BuildError(format!(
                "{} does not compile:\n{}",
                source.shown(),
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
#[derive(Clone, Copy)]
enum Part {
    Solution,
    ColdCache,
    Brute,
    /// The `@gen` fn, called with this size.
    Gen(i64),
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
    let errors = |diagnostics: &[cpl_diag::Diagnostic]| {
        BuildError(format!(
            "{name} does not build:\n{}",
            cpl_diag::render_all(diagnostics, &name, &text).trim_end()
        ))
    };
    let mut db = cpl_check::Database::new();
    db.set_file(&name, text.as_str());
    let analysis = db.analyze(&name).expect("the file was just set");
    let is_error = |d: &cpl_diag::Diagnostic| d.severity == cpl_diag::Severity::Error;
    if analysis.diagnostics.iter().any(is_error) {
        return Err(errors(&analysis.diagnostics));
    }
    let missing = |kind: &str| BuildError(format!("{name} declares no `{kind}` fn"));
    let lowered = match part {
        Part::Solution => cpl_ir::lower_sampling_laws(&analysis),
        Part::ColdCache => cpl_ir::lower_cold_cache(&analysis),
        Part::Brute => {
            let (brute_fn, _) = cpl_ir::brute(&analysis).ok_or_else(|| missing("@alt(program)"))?;
            cpl_ir::lower_brute(&analysis, &brute_fn)
        }
        Part::Gen(size) => {
            let (gen_fn, _) = cpl_ir::generator(&analysis).ok_or_else(|| missing("@gen"))?;
            cpl_ir::lower_gen(&analysis, &gen_fn, size)
        }
    };
    let program = lowered.map_err(|diagnostics| errors(&diagnostics))?;
    let file = source.file_name().unwrap_or_default().to_string_lossy();
    let target = cpl_codegen::Target {
        stamp: Some(profile.stamp()),
        thread_stack: profile.thread_stack,
        windows: profile.os == cpl_judge::Os::Windows,
        checks,
    };
    let mut cpp = cpl_codegen::emit_for(&program, &file, None, &target);
    if matches!(part, Part::Gen(_)) {
        // Its `rand` and `shuffle` draw from the choices `cpl stress` keeps.
        cpp.insert_str(0, "#define CPL_GEN 1\n");
    }
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| BuildError(format!("can't create {}: {e}", dir.shown())))?;
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let out = dir.join(format!("{stem}-{:016x}.cpp", hash(cpp.as_bytes())));
    std::fs::write(&out, cpp)
        .map_err(|e| BuildError(format!("can't write {}: {e}", out.shown())))?;
    Ok((out, profile))
}

/// Builds the C++ of a `.cp` file as its judge does, as far as the
/// compiler here allows, and runs it on the judge's stack.
fn build_judged(
    source: &Path,
    text: Option<&str>,
    part: Part,
    checks: cpl_codegen::Checks,
) -> Result<Vec<String>, BuildError> {
    let (cpp, profile) = emit_cpp(source, text, part, checks)?;
    let host = profile.host();
    if !found(&host.compiler) {
        return interpreted(source, text, part, &host.compiler);
    }
    let binary = compile_with(&cpp, true, &host.compiler, &host.flags)?;
    let stack = profile.ulimit();
    // Off Unix there is no `ulimit`: the stack is the one the linker gave.
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

/// Whether `compiler` can be started here, asked once per compiler.
fn found(compiler: &str) -> bool {
    type Asked = std::sync::Mutex<std::collections::HashMap<String, bool>>;
    static FOUND: std::sync::OnceLock<Asked> = std::sync::OnceLock::new();
    let mut found = FOUND.get_or_init(Default::default).lock().unwrap();
    *found.entry(compiler.to_string()).or_insert_with(|| {
        let probe = Command::new(compiler)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        !matches!(probe, Err(e) if e.kind() == std::io::ErrorKind::NotFound)
    })
}

/// `part` of `source` run by `cpl run` in the interpreter, on a machine
/// with no C++ compiler: slower, but the same answers.
fn interpreted(
    source: &Path,
    text: Option<&str>,
    part: Part,
    compiler: &str,
) -> Result<Vec<String>, BuildError> {
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
        Part::Gen(_) => {
            return Err(BuildError(format!(
                "there is no {compiler} here to build the `@gen` fn of {}; install it or pass `--gen <program>`",
                source.shown()
            )))
        }
    }
    argv.push(source.display().to_string());
    Ok(argv)
}

/// The `ulimit -s` programs started from here get.
fn current_stack() -> Option<String> {
    static STACK: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    STACK
        .get_or_init(|| {
            let out = Command::new("sh").args(["-c", "ulimit -s"]).output().ok()?;
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        })
        .clone()
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
    let name = interpreter_name(program);
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
            env: Vec::new(),
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
            env: Vec::new(),
        })
    }

    /// The `.cp` file at `path` built for its judge with `checks` instead
    /// of the traps stress keeps (`cpl run --release --checks`).
    pub fn resolve_checks(path: &Path, checks: cpl_codegen::Checks) -> Result<Program, BuildError> {
        Ok(Program {
            name: path.shown(),
            argv: build_judged(path, None, Part::Solution, checks)?,
            env: Vec::new(),
        })
    }

    /// The `.cp` source `text`, built as if it were the file `path`, which
    /// need not exist: its imports and judge resolve from beside `path`.
    pub fn resolve_source(path: &Path, text: &str) -> Result<Program, BuildError> {
        Ok(Program {
            name: path.shown(),
            argv: build_judged(path, Some(text), Part::Solution, LOCAL)?,
            env: Vec::new(),
        })
    }

    /// The solution in `@cache` cold mode (cache writes through read `self`
    /// are undone at method exit), to check P1 against the normal run.
    pub fn resolve_cold_cache(path: &Path) -> Result<Program, BuildError> {
        Ok(Program {
            name: format!("{} (@cache cold)", path.shown()),
            argv: build_judged(path, None, Part::ColdCache, LOCAL)?,
            env: Vec::new(),
        })
    }

    /// The `@alt(program)` fn of the `.cp` file `path`, built like the
    /// solution: the file's input header, then the fn.
    pub fn resolve_brute(path: &Path, fn_name: &str) -> Result<Program, BuildError> {
        Ok(Program {
            name: format!("{} (`{fn_name}`)", path.shown()),
            argv: build_judged(path, None, Part::Brute, LOCAL)?,
            env: Vec::new(),
        })
    }

    /// The `@gen` fn of the `.cp` file `path`, built as a program that
    /// prints one input of at most `size`.
    pub fn resolve_gen(path: &Path, fn_name: &str, size: i64) -> Result<Program, BuildError> {
        Ok(Program {
            name: format!("{} (`{fn_name}`)", path.shown()),
            argv: build_judged(path, None, Part::Gen(size), LOCAL)?,
            env: Vec::new(),
        })
    }

    /// The `@gen` fn of the `.cp` file `path` lowered to print one input of
    /// at most `size` in the interpreter, when no C++ compiler is here to
    /// build it; `None` when one is.
    pub fn interpreted_gen(path: &Path, size: i64) -> Result<Option<cpl_ir::Program>, BuildError> {
        let name = path.shown();
        let text = std::fs::read_to_string(path)
            .map_err(|e| BuildError(format!("can't read {name}: {e}")))?;
        let profile = cpl_judge::resolve(path, &text, None).map_err(BuildError)?;
        if found(&profile.host().compiler) {
            return Ok(None);
        }
        Self::lowered_gen(path, size).map(Some)
    }

    /// The file's `@gen` fn lowered for the interpreter, compiler or not.
    pub fn lowered_gen(path: &Path, size: i64) -> Result<cpl_ir::Program, BuildError> {
        let name = path.shown();
        let text = std::fs::read_to_string(path)
            .map_err(|e| BuildError(format!("can't read {name}: {e}")))?;
        let mut db = cpl_check::Database::new();
        db.set_file(&name, text.as_str());
        let analysis = db.analyze(&name).expect("the file was just set");
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
        cpl_ir::lower_gen(&analysis, &gen_fn, size).map_err(|diagnostics| failed(&diagnostics))
    }

    /// The command with `extra` arguments added.
    pub fn with_args(&self, extra: &[&str]) -> Program {
        let mut program = self.clone();
        program.argv.extend(extra.iter().map(|arg| arg.to_string()));
        program
    }

    pub(crate) fn to_command(&self) -> Command {
        let mut command = Command::new(&self.argv[0]);
        command.args(&self.argv[1..]);
        command.envs(self.env.iter().map(|(key, value)| (key, value)));
        // Its own process group, so a time limit stops what it started too.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
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
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let pid = child.id();
        let (out, err, exit, peak_kb) = std::thread::scope(|scope| {
            // Each pipe has its own thread, so a program that prints a
            // lot or never reads its input cannot block either side.
            scope.spawn(move || {
                let _ = stdin.write_all(input.as_bytes());
            });
            let out = scope.spawn(move || {
                let mut buffer = Vec::new();
                let _ = stdout.read_to_end(&mut buffer);
                buffer
            });
            let err = scope.spawn(move || {
                let mut buffer = Vec::new();
                let _ = stderr.read_to_end(&mut buffer);
                buffer
            });
            let mut peak = None;
            let exit = loop {
                peak = peak_memory(pid).max(peak);
                match child.try_wait() {
                    Ok(Some(status)) => break status.code().map_or(Exit::Signal, Exit::Code),
                    Ok(None) if start.elapsed() >= limit => {
                        kill_group(pid);
                        let _ = child.kill();
                        let _ = child.wait();
                        break Exit::TimedOut;
                    }
                    Ok(None) => std::thread::sleep(Duration::from_micros(300)),
                    Err(_) => break Exit::Signal,
                }
            };
            (out.join().unwrap(), err.join().unwrap(), exit, peak)
        });
        Ok(Outcome {
            // LF everywhere: a Windows program's `\r\n` never reaches a compare, report or saved case.
            stdout: String::from_utf8_lossy(&out).replace("\r\n", "\n"),
            stderr: String::from_utf8_lossy(&err).replace("\r\n", "\n"),
            exit,
            elapsed: start.elapsed(),
            peak_kb,
        })
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

/// Stops a program and everything it started. A shell script's children
/// would otherwise keep its pipes open after the script is killed.
pub(crate) fn kill_group(pid: u32) {
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    // `py -3` runs the script in a child python.exe, which `Child::kill`
    // leaves running with the pipes open.
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(not(any(unix, windows)))]
    let _ = pid;
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
