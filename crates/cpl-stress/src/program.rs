//! Something that can be run on an input: a solution, a brute, a
//! generator, a checker.
//!
//! A path becomes a program by its extension: `.cpp` is compiled with
//! `g++ -std=c++17 -O2` (once; the binary is cached by content), `.py`
//! runs under `python3`, and anything else is run as it is. A `.cp` file
//! goes through the compiler's C++ back end first, the same one behind
//! `cpl build`.

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
    let text = std::fs::read(source)
        .map_err(|e| BuildError(format!("can't read {}: {e}", source.display())))?;
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| BuildError(format!("can't create {}: {e}", dir.display())))?;
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let binary = dir.join(format!("{stem}-{:016x}", hash(&text)));
    if !binary.is_file() {
        // Built under another name, then renamed, so a build that is
        // interrupted or raced never leaves half a binary behind.
        let partial = dir.join(format!(
            "{stem}-{:016x}.{}.tmp",
            hash(&text),
            std::process::id()
        ));
        let mut build = Command::new("g++");
        build.args(CXX_FLAGS);
        if ours || starts_with_std(&text) {
            if let Some(header) = std_header() {
                build.arg("-include").arg(header);
            }
        }
        let output = build
            .arg("-o")
            .arg(&partial)
            .arg(source)
            .output()
            .map_err(|e| BuildError(format!("can't run g++ to build {}: {e}", source.display())))?;
        if !output.status.success() {
            let errors = String::from_utf8_lossy(&output.stderr);
            let shown: Vec<&str> = errors.lines().take(20).collect();
            return Err(BuildError(format!(
                "{} does not compile:\n{}",
                source.display(),
                shown.join("\n")
            )));
        }
        std::fs::rename(&partial, &binary)
            .map_err(|e| BuildError(format!("can't write {}: {e}", binary.display())))?;
    }
    Ok(binary.display().to_string())
}

/// Emits the C++ for a `.cp` file into the cache, through the same front
/// end and back end as `cpl build`. Returns the path of the C++ file.
fn emit_cpp(source: &Path, brute: bool) -> Result<PathBuf, BuildError> {
    let name = source.display().to_string();
    let text = std::fs::read_to_string(source)
        .map_err(|e| BuildError(format!("can't read {name}: {e}")))?;
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
    let lowered = match cpl_ir::brute(&analysis) {
        Some((brute_fn, _)) if brute => cpl_ir::lower_brute(&analysis, &brute_fn),
        None if brute => return Err(BuildError(format!("{name} declares no `@brute(main)` fn"))),
        _ => cpl_ir::lower(&analysis),
    };
    let program = lowered.map_err(|diagnostics| errors(&diagnostics))?;
    let file = source.file_name().unwrap_or_default().to_string_lossy();
    let cpp = cpl_codegen::emit(&program, &file, None);
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| BuildError(format!("can't create {}: {e}", dir.display())))?;
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let out = dir.join(format!("{stem}-{:016x}.cpp", hash(cpp.as_bytes())));
    std::fs::write(&out, cpp)
        .map_err(|e| BuildError(format!("can't write {}: {e}", out.display())))?;
    Ok(out)
}

impl Program {
    /// A program that runs `argv` as given.
    pub fn command(name: &str, argv: &[&str]) -> Program {
        Program {
            name: name.to_string(),
            argv: argv.iter().map(|arg| arg.to_string()).collect(),
        }
    }

    /// Makes `path` runnable, building it if it is source.
    pub fn resolve(path: &Path) -> Result<Program, BuildError> {
        let name = path.display().to_string();
        if !path.is_file() {
            return Err(BuildError(format!("there is no file {name}")));
        }
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let argv = match extension {
            "cpp" | "cc" | "cxx" => vec![compile_cpp(path, false)?],
            "py" => vec!["python3".to_string(), name.clone()],
            "cp" => vec![compile_cpp(&emit_cpp(path, false)?, true)?],
            // Relative paths need `./` to be found.
            _ => vec![path
                .canonicalize()
                .map_err(|e| BuildError(format!("can't find {name}: {e}")))?
                .display()
                .to_string()],
        };
        Ok(Program { name, argv })
    }

    /// The `@brute(main)` fn of the `.cp` file `path`, built like the
    /// solution: the file's input header, then the fn.
    pub fn resolve_brute(path: &Path, fn_name: &str) -> Result<Program, BuildError> {
        Ok(Program {
            name: format!("{} (`{fn_name}`)", path.display()),
            argv: vec![compile_cpp(&emit_cpp(path, true)?, true)?],
        })
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
            stdout: String::from_utf8_lossy(&out).into_owned(),
            stderr: String::from_utf8_lossy(&err).into_owned(),
            exit,
            elapsed: start.elapsed(),
            peak_kb,
        })
    }
}

/// The high-water mark of a running process's memory, in kilobytes.
fn peak_memory(pid: u32) -> Option<u64> {
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
    #[cfg(not(unix))]
    let _ = pid;
}

#[cfg(test)]
mod tests {
    use super::starts_with_std;

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
