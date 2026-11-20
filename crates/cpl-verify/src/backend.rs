//! The things under test, behind two small traits.
//!
//! A [`Transpiler`] turns `.cp` source into C++17 and an [`Interpreter`]
//! runs `.cp` source directly. Today the transpiler is the Python
//! prototype in `proto/cpc/`; `cpl-codegen` and `cpl-interp` plug in
//! through the same traits, either as a `cpl` binary ([`CommandTranspiler`],
//! [`CommandInterpreter`]) or, once they export an API, in process.

use crate::process::{self, Output};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const COMPILE_LIMIT: Duration = Duration::from_secs(120);

pub trait Transpiler: Sync {
    fn name(&self) -> String;
    /// Writes the C++ for `source` to `cpp`. `Err` is the diagnostics.
    fn transpile(&self, source: &Path, cpp: &Path) -> Result<(), String>;
}

pub trait Interpreter: Sync {
    fn name(&self) -> String;
    fn run(&self, source: &Path, stdin: &[u8], limit: Duration) -> std::io::Result<Output>;
}

/// The repository root: the nearest ancestor of this crate, or of the
/// current directory, that holds `proto/cp.py`.
pub fn repo_root() -> Option<PathBuf> {
    let from_crate = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    let from_cwd = std::env::current_dir().ok();
    [Some(from_crate), from_cwd]
        .into_iter()
        .flatten()
        .flat_map(|start| start.ancestors().map(Path::to_path_buf).collect::<Vec<_>>())
        .find(|dir| dir.join("proto/cp.py").is_file())
}

/// `python3 proto/cp.py build`.
pub struct ProtoTranspiler {
    script: PathBuf,
}

impl ProtoTranspiler {
    pub fn new(repo_root: &Path) -> Self {
        ProtoTranspiler {
            script: repo_root.join("proto/cp.py"),
        }
    }
}

impl Transpiler for ProtoTranspiler {
    fn name(&self) -> String {
        "proto/cpc".into()
    }

    fn transpile(&self, source: &Path, cpp: &Path) -> Result<(), String> {
        // `cp.py build x.cp` writes `x.cpp` beside the source and stamps
        // the source with its version, so it only ever sees a copy.
        let dir = cpp.parent().ok_or("the C++ path has no directory")?;
        let copy = dir.join("proto_input.cp");
        std::fs::copy(source, &copy).map_err(|e| format!("copying the source: {e}"))?;
        let out = process::run(
            Command::new("python3")
                .arg(&self.script)
                .arg("build")
                .arg(&copy),
            b"",
            COMPILE_LIMIT,
        )
        .map_err(|e| format!("running python3: {e}"))?;
        if !out.ok() {
            return Err(format!("{}{}", out.stdout, out.stderr));
        }
        std::fs::rename(copy.with_extension("cpp"), cpp).map_err(|e| format!("moving the C++: {e}"))
    }
}

/// Fills `{src}` and `{cpp}` in a command line.
fn fill(template: &[String], source: &Path, cpp: Option<&Path>) -> Command {
    let subst = |arg: &String| {
        let arg = arg.replace("{src}", &source.to_string_lossy());
        match cpp {
            Some(cpp) => arg.replace("{cpp}", &cpp.to_string_lossy()),
            None => arg,
        }
    };
    let mut cmd = Command::new(subst(&template[0]));
    cmd.args(template[1..].iter().map(subst));
    cmd
}

/// Any command that writes C++ for `{src}` to `{cpp}`, for example
/// `cpl build {src} --emit-cpp {cpp}`.
pub struct CommandTranspiler {
    pub template: Vec<String>,
}

impl Transpiler for CommandTranspiler {
    fn name(&self) -> String {
        self.template.join(" ")
    }

    fn transpile(&self, source: &Path, cpp: &Path) -> Result<(), String> {
        let out = process::run(
            &mut fill(&self.template, source, Some(cpp)),
            b"",
            COMPILE_LIMIT,
        )
        .map_err(|e| format!("running {}: {e}", self.template[0]))?;
        if !out.ok() {
            return Err(format!("{}{}", out.stdout, out.stderr));
        }
        if !cpp.is_file() {
            return Err(format!("`{}` wrote no C++ to {{cpp}}", self.name()));
        }
        Ok(())
    }
}

/// Any command that runs `{src}`, for example `cpl run {src}`.
pub struct CommandInterpreter {
    pub template: Vec<String>,
}

impl Interpreter for CommandInterpreter {
    fn name(&self) -> String {
        self.template.join(" ")
    }

    fn run(&self, source: &Path, stdin: &[u8], limit: Duration) -> std::io::Result<Output> {
        process::run(&mut fill(&self.template, source, None), stdin, limit)
    }
}

/// A C++ compiler invocation, with an optional precompiled
/// `<bits/stdc++.h>` built with exactly the same flags.
#[derive(Debug, Clone)]
pub struct Cxx {
    pub compiler: String,
    pub flags: Vec<String>,
    pch_dir: Option<PathBuf>,
}

impl Cxx {
    fn with_flags(flags: &[&str]) -> Self {
        Cxx {
            compiler: std::env::var("CXX").unwrap_or_else(|_| "g++".into()),
            flags: flags.iter().map(|s| s.to_string()).collect(),
            pch_dir: None,
        }
    }

    /// `g++ -std=c++17 -O2 -fsanitize=address,undefined`: the build every
    /// fuzzed program runs under.
    pub fn sanitized() -> Self {
        Self::with_flags(&["-std=c++17", "-O2", "-fsanitize=address,undefined"])
    }

    /// `g++ -std=c++17 -O2`: what `cpl run --release` and a judge use.
    pub fn release() -> Self {
        Self::with_flags(&["-std=c++17", "-O2"])
    }

    pub fn describe(&self) -> String {
        let pch = if self.pch_dir.is_some() { " (PCH)" } else { "" };
        format!("{} {}{pch}", self.compiler, self.flags.join(" "))
    }

    /// Precompiles `<bits/stdc++.h>` into `dir` and uses it from now on.
    pub fn build_pch(&mut self, dir: &Path) -> Result<Duration, String> {
        let bits = dir.join("bits");
        std::fs::create_dir_all(&bits).map_err(|e| format!("creating {}: {e}", bits.display()))?;
        let header = dir.join("pch_source.h");
        std::fs::write(&header, "#include <bits/stdc++.h>\n").map_err(|e| e.to_string())?;
        let out = process::run(
            Command::new(&self.compiler)
                .args(&self.flags)
                .args(["-x", "c++-header"])
                .arg(&header)
                .arg("-o")
                .arg(bits.join("stdc++.h.gch")),
            b"",
            COMPILE_LIMIT,
        )
        .map_err(|e| format!("running {}: {e}", self.compiler))?;
        if !out.ok() {
            return Err(out.stderr);
        }
        self.pch_dir = Some(dir.to_path_buf());
        Ok(out.elapsed)
    }

    /// Compiles `cpp` to `exe`. `Err` is the compiler's diagnostics.
    pub fn compile(&self, cpp: &Path, exe: &Path) -> Result<Duration, String> {
        let mut cmd = Command::new(&self.compiler);
        cmd.args(&self.flags);
        if let Some(dir) = &self.pch_dir {
            // An unusable PCH must fail loudly, not silently fall back to
            // the slow path.
            cmd.arg("-I")
                .arg(dir)
                .arg("-Winvalid-pch")
                .arg("-Werror=invalid-pch");
        }
        let out = process::run(cmd.arg("-o").arg(exe).arg(cpp), b"", COMPILE_LIMIT)
            .map_err(|e| format!("running {}: {e}", self.compiler))?;
        if out.ok() {
            Ok(out.elapsed)
        } else {
            Err(out.stderr)
        }
    }
}

/// True when `stderr` holds an AddressSanitizer, LeakSanitizer or
/// UndefinedBehaviorSanitizer report.
pub fn sanitizer_report(stderr: &str) -> Option<&str> {
    stderr.lines().find(|line| {
        line.contains("runtime error:")
            || line.contains("ERROR: AddressSanitizer")
            || line.contains("ERROR: LeakSanitizer")
            || line.contains("UndefinedBehaviorSanitizer")
    })
}

/// A fresh, empty directory under the system temp dir.
pub fn scratch_dir(label: &str) -> std::io::Result<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "cpl-verify-{}-{}-{label}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_sanitizer_reports() {
        let ubsan =
            "t.cpp:185:16: runtime error: signed integer overflow: 1 + 2 cannot be represented";
        let asan = "==8938==ERROR: AddressSanitizer: heap-use-after-free on address 0x503000000048";
        assert!(sanitizer_report(ubsan).is_some());
        assert!(sanitizer_report(asan).is_some());
        assert!(sanitizer_report("line 3: list index out of range").is_none());
    }

    #[test]
    fn fills_placeholders() {
        let template = ["cpl", "build", "{src}", "--emit-cpp={cpp}"].map(String::from);
        let cmd = fill(&template, Path::new("a.cp"), Some(Path::new("out/a.cpp")));
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["build", "a.cp", "--emit-cpp=out/a.cpp"]);
    }
}
