//! Something that can be run on an input: a solution, a brute, a
//! generator, a checker.
//!
//! A path becomes a program by its extension: `.cpp` is compiled with
//! `g++ -std=c++17 -O2` (once; the binary is cached by content), `.py`
//! runs under `python3`, and anything else is run as it is. A `.cp` file
//! needs the compiler's C++ backend, which is not written yet; with
//! `CPL_PROTO` set to the prototype's `cp.py` it is built through that.

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

fn compile_cpp(source: &Path) -> Result<String, BuildError> {
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
        let output = Command::new("g++")
            .args(["-std=c++17", "-O2", "-o"])
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
            "cpp" | "cc" | "cxx" => vec![compile_cpp(path)?],
            "py" => vec!["python3".to_string(), name.clone()],
            "cp" => {
                let Some(proto) = std::env::var_os("CPL_PROTO") else {
                    return Err(BuildError(format!(
                        "cpl can't build {name} yet: the C++ backend is not written. Pass the built solution (`{}`), or set CPL_PROTO to the prototype's cp.py",
                        path.with_extension("cpp").display()
                    )));
                };
                let output = Command::new("python3")
                    .arg(&proto)
                    .arg("build")
                    .arg(path)
                    .output()
                    .map_err(|e| BuildError(format!("can't run the prototype compiler: {e}")))?;
                if !output.status.success() {
                    return Err(BuildError(format!(
                        "{name} does not build:\n{}",
                        String::from_utf8_lossy(&output.stderr).trim_end()
                    )));
                }
                vec![compile_cpp(&path.with_extension("cpp"))?]
            }
            // Relative paths need `./` to be found.
            _ => vec![path
                .canonicalize()
                .map_err(|e| BuildError(format!("can't find {name}: {e}")))?
                .display()
                .to_string()],
        };
        Ok(Program { name, argv })
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
