//! `cpl build`: one C++17 file for the judge, its size report, and
//! `--checked`, the same file built here with the sanitizers on.

use std::io::Write;

use crate::submit::{copy_to_clipboard, size_report};
use crate::{compile, EXIT_ERRORS, EXIT_USAGE};
use cpl_diag::Shown;

/// What `cpl build` is asked for beyond the file: `out` says where the
/// C++ goes, next to the source when `None` (`-` is standard output).
#[derive(Clone, Copy, Default)]
pub(crate) struct BuildOptions<'a> {
    pub out: Option<&'a str>,
    pub judge: Option<&'a str>,
    pub checked: bool,
    pub checks: cpl_codegen::Checks,
    pub size: bool,
    pub copy: bool,
    pub json: bool,
}

/// `cpl build`: one C++17 file for the judge.
pub(crate) fn build_file(
    path: &str,
    options: &BuildOptions,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let BuildOptions {
        out,
        judge,
        checked,
        checks,
        size,
        copy,
        json,
    } = *options;
    if checked && out == Some("-") {
        *stderr += "cpl: `--checked` builds a program from the C++ file, so it can't go to standard output\n";
        return EXIT_USAGE;
    }
    if json && (checked || out == Some("-")) {
        *stderr += "cpl: `--json` prints the report on standard output, so it goes without `--checked` and `-o -`\n";
        return EXIT_USAGE;
    }
    let compiled = match compile(path, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let profile = match cpl_judge::resolve(std::path::Path::new(path), &compiled.text, judge) {
        Ok(profile) => profile,
        Err(error) => {
            *stderr += &format!("cpl: {error}\n");
            // `--judge nope` is misuse; a `// judge:` line or cp.toml is the files'.
            return if judge.is_some() {
                EXIT_USAGE
            } else {
                EXIT_ERRORS
            };
        }
    };
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    let limit = profile.limit();
    let target = cpl_codegen::Target {
        stamp: Some(profile.stamp()),
        thread_stack: profile.thread_stack,
        windows: profile.os == cpl_judge::Os::Windows,
        checks,
    };
    let mut sizes = Vec::new();
    let aim = profile.aim();
    let cpp = match cpl_codegen::emit_aiming(
        &compiled.program,
        name,
        &compiled.text,
        (aim, limit),
        &target,
    ) {
        Ok(fitted) => {
            if let Some(note) = fitted.note(path, &format!("{aim}-byte target"), &profile.name) {
                *stderr += &size_warning(&note, path, &compiled.text);
            }
            if size || json {
                let source = fitted.header.then_some(compiled.text.as_str());
                sizes = cpl_codegen::sizes(&compiled.program, name, source, &target, &fitted);
            }
            if size {
                *stderr += &size_report(&sizes, limit, &profile.name);
            }
            fitted.cpp
        }
        Err(shortest) => {
            *stderr += &format!(
                "cpl: the C++ for `{path}` is {} bytes, over the {limit}-byte limit of judge `{}`\n",
                shortest.cpp.len(),
                profile.name
            );
            let sizes = cpl_codegen::sizes(&compiled.program, name, None, &target, &shortest);
            *stderr += &size_report(&sizes, limit, &profile.name);
            return EXIT_ERRORS;
        }
    };
    if let Some(problem) = profile.unsupported(&cpp) {
        *stderr += &format!("cpl: `{path}` can't be submitted as it is: {problem}\n");
        return EXIT_ERRORS;
    }
    if let Some(warning) = profile.thread_warning() {
        *stderr += &format!("cpl: warning: {warning}\n");
    }
    if cpp.len() > profile.warn_at() {
        let near = format!("{} bytes, near ", profile.warn_at());
        let over = if cpp.len() > aim { "" } else { &near };
        let warning = format!(
            "the C++ for `{path}` is {} bytes, over {over}the {aim}-byte target and {}% of the {limit}-byte limit of judge `{}`; \
             `--size` says where the bytes go",
            cpp.len(),
            cpp.len() * 100 / limit,
            profile.name
        );
        *stderr += &size_warning(&warning, path, &compiled.text);
    }
    if copy {
        match copy_to_clipboard(&cpp) {
            Ok(tool) => {
                *stderr += &format!(
                    "cpl: copied the C++ ({} bytes) to the clipboard with `{tool}`\n",
                    cpp.len()
                )
            }
            Err(error) => {
                *stderr += &format!("cpl: warning: {error}; the C++ is only in the file\n")
            }
        }
    }
    if out == Some("-") {
        let _ = stdout.write_all(cpp.as_bytes());
        return 0;
    }
    let target = match out {
        Some(out) => std::path::PathBuf::from(out),
        None => std::path::Path::new(path).with_extension("cpp"),
    };
    match std::fs::write(&target, &cpp) {
        Ok(()) if json => {
            let sizes = sizes.iter().map(|size| {
                cpl_json::obj([
                    ("part", size.part.describe().into()),
                    ("name", size.name.as_str().into()),
                    ("bytes", size.bytes.into()),
                ])
            });
            let report = cpl_json::obj([
                ("path", target.shown().into()),
                ("bytes", cpp.len().into()),
                ("limit", limit.into()),
                ("judge", profile.name.to_string().into()),
                ("checks", checks.name().to_string().into()),
                ("sizes", cpl_json::Value::Array(sizes.collect())),
            ]);
            let _ = writeln!(stdout, "{}", cpl_json::to_string(&report));
            0
        }
        Ok(()) => {
            let _ = writeln!(
                stdout,
                "wrote {} ({} bytes, {}% of the limit of judge `{}`)",
                target.shown(),
                cpp.len(),
                (cpp.len() * 100).div_ceil(limit),
                profile.name
            );
            if checked {
                return build_checked(&target, stdout, stderr);
            }
            0
        }
        Err(error) => {
            *stderr += &format!("cpl: can't write `{}`: {error}\n", target.shown());
            EXIT_USAGE
        }
    }
}

/// A size warning, as a W0201 diagnostic on the program's first line,
/// below the stamp and header comments.
fn size_warning(message: &str, path: &str, text: &str) -> String {
    let message = message.trim_start_matches("cpl: ").trim_end();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let code = line.trim();
        if !code.is_empty() && !code.starts_with("//") {
            break;
        }
        at += line.len();
    }
    let at = if at == text.len() { 0 } else { at as u32 };
    let warning = cpl_diag::Diagnostic::warning(
        cpl_diag::codes::SIZE_NEAR_LIMIT,
        cpl_diag::Span::new(at, at),
        message,
    );
    cpl_diag::render_all(&[warning], path, text)
}

/// Removes `--judge <name>` from `rest`, returning the name.
pub(crate) fn take_judge<'a>(
    rest: &mut Vec<&'a str>,
    command: &str,
) -> Result<Option<&'a str>, String> {
    match rest.iter().position(|arg| *arg == "--judge") {
        Some(at) if at + 1 < rest.len() => {
            let name = rest[at + 1];
            rest.drain(at..at + 2);
            Ok(Some(name))
        }
        Some(_) => Err(format!(
            "`--judge` takes a judge's name: `cpl {command} a.cp --judge codeforces`"
        )),
        None => Ok(None),
    }
}

/// What a judge build checks unless `--checks` says: hand-written C++'s
/// checks, since on the judge a crash and a wrong answer cost the same
/// (decisions.md 299).
pub(crate) const JUDGE_CHECKS: cpl_codegen::Checks = cpl_codegen::Checks::None;

/// `--checks <level>`, taken out of `rest`. Where it is absent,
/// [`JUDGE_CHECKS`], or `traps` for a `--checked` build, which is built to
/// stop where the program goes wrong.
pub(crate) fn take_checks(
    rest: &mut Vec<&str>,
    checked: bool,
) -> Result<cpl_codegen::Checks, String> {
    let Some(at) = rest.iter().position(|arg| *arg == "--checks") else {
        return Ok(match checked {
            true => cpl_codegen::Checks::Traps,
            false => JUDGE_CHECKS,
        });
    };
    let level = rest
        .get(at + 1)
        .and_then(|name| cpl_codegen::Checks::parse(name));
    let Some(level) = level else {
        return Err(format!(
            "`--checks` takes one of {}: `cpl build a.cp --checks none`",
            cpl_codegen::Checks::NAMES
                .map(|n| format!("`{n}`"))
                .join(", ")
        ));
    };
    rest.drain(at..at + 2);
    Ok(level)
}

/// `cpl build --checked`: the C++ file built to run here, with the
/// sanitizers on, so overflow, a bad index or a bad shift stops the
/// program with a report instead of going on with a wrong value.
fn build_checked(
    cpp: &std::path::Path,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    // `LOCAL` turns on the checks the sanitizers have no word for: a
    // negative power, an `I128` shifted past its top, a `U64` divided by 0.
    const FLAGS: [&str; 7] = [
        "-std=c++17",
        "-O1",
        "-g",
        "-fsanitize=address,undefined",
        "-fno-sanitize-recover=all",
        "-D_GLIBCXX_ASSERTIONS",
        "-DLOCAL",
    ];
    // MinGW (Windows' g++) has no sanitizer runtime to link: there the
    // undefined-behaviour checks trap in place, with no address checks.
    const TRAPPING: [&str; 7] = [
        "-std=c++17",
        "-O1",
        "-g",
        "-fsanitize=undefined",
        "-fsanitize-undefined-trap-on-error",
        "-D_GLIBCXX_ASSERTIONS",
        "-DLOCAL",
    ];
    let binary = cpp.with_extension("checked");
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "g++".into());
    let build = |flags: &[&str]| {
        std::process::Command::new(&compiler)
            .args(flags)
            .arg("-o")
            .arg(&binary)
            .arg(cpp)
            .output()
    };
    let mut flags = &FLAGS;
    let mut built = build(flags);
    if let Ok(output) = &built {
        if !output.status.success()
            && no_sanitizer_runtime(&String::from_utf8_lossy(&output.stderr))
        {
            flags = &TRAPPING;
            built = build(flags);
        }
    }
    match built {
        Ok(output) if output.status.success() => {
            let _ = writeln!(
                stdout,
                "built {} ({compiler} {})",
                binary.shown(),
                flags.join(" ")
            );
            if flags == &TRAPPING {
                *stderr += &format!(
                    "cpl: note: {compiler} has no sanitizer runtime, so this build traps on undefined behaviour but has no address checks\n"
                );
            }
            0
        }
        Ok(output) => {
            let errors = String::from_utf8_lossy(&output.stderr);
            let shown: Vec<&str> = errors.lines().take(20).collect();
            *stderr += &format!(
                "cpl: {compiler} could not build `{}`:\n{}\n",
                cpp.shown(),
                shown.join("\n")
            );
            EXIT_ERRORS
        }
        Err(error) => {
            *stderr += &format!(
                "cpl: `--checked` needs a C++ compiler, and `{compiler}` did not start: {error}\n"
            );
            EXIT_USAGE
        }
    }
}

/// Whether a failed sanitized build failed only because the compiler has
/// no sanitizer runtime to link (`cannot find -lasan` on MinGW).
fn no_sanitizer_runtime(errors: &str) -> bool {
    ["-lasan", "-lubsan", "libasan", "libubsan"]
        .iter()
        .any(|lib| errors.contains(lib))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_missing_sanitizer_runtime_is_told_from_a_compile_error() {
        let mingw = "ld.exe: cannot find -lasan: No such file or directory\nld.exe: cannot find -lubsan: No such file or directory\ncollect2.exe: error: ld returned 1 exit status\n";
        assert!(super::no_sanitizer_runtime(mingw));
        assert!(!super::no_sanitizer_runtime(
            "a.cpp:3:5: error: 'x' was not declared in this scope\n"
        ));
    }
}
