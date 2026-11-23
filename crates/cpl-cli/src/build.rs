//! `cpl build`: one C++17 file for the judge, its size report, and
//! `--checked`, the same file built here with the sanitizers on.

use std::io::Write;

use crate::submit::{copy_to_clipboard, size_report};
use crate::{compile, EXIT_ERRORS, EXIT_USAGE};

/// What `cpl build` is asked for beyond the file: `out` says where the
/// C++ goes, next to the source when `None` (`-` is standard output).
#[derive(Clone, Copy, Default)]
pub(crate) struct BuildOptions<'a> {
    pub out: Option<&'a str>,
    pub judge: Option<&'a str>,
    pub checked: bool,
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
            return EXIT_USAGE;
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
    };
    let mut sizes = Vec::new();
    let cpp = match cpl_codegen::emit_within_for(
        &compiled.program,
        name,
        &compiled.text,
        limit,
        &target,
    ) {
        Ok(fitted) => {
            *stderr += &fitted.note(path, limit, &profile.name).unwrap_or_default();
            if size || json {
                let source = fitted.header.then_some(compiled.text.as_str());
                sizes = cpl_codegen::sizes(
                    &compiled.program,
                    name,
                    source,
                    &target,
                    (fitted.comments, fitted.narrow),
                    fitted.cpp.len(),
                );
            }
            if size {
                *stderr += &size_report(&sizes, limit, &profile.name);
            }
            fitted.cpp
        }
        Err(bytes) => {
            *stderr += &format!(
                "cpl: the C++ for `{path}` is {bytes} bytes, over the {limit}-byte limit of judge `{}`\n",
                profile.name
            );
            let sizes =
                cpl_codegen::sizes(&compiled.program, name, None, &target, (false, true), bytes);
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
        *stderr += &format!(
            "cpl: warning: the C++ for `{path}` is {} bytes, over the {}-byte target and {}% of the {limit}-byte \
             limit of judge `{}`; `--size` says where the bytes go\n",
            cpp.len(),
            profile.warn_at(),
            cpp.len() * 100 / limit,
            profile.name
        );
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
                ("path", target.display().to_string().into()),
                ("bytes", cpp.len().into()),
                ("limit", limit.into()),
                ("judge", profile.name.to_string().into()),
                ("sizes", cpl_json::Value::Array(sizes.collect())),
            ]);
            let _ = writeln!(stdout, "{}", cpl_json::to_string(&report));
            0
        }
        Ok(()) => {
            let _ = writeln!(
                stdout,
                "wrote {} ({} bytes, {}% of the limit of judge `{}`)",
                target.display(),
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
            *stderr += &format!("cpl: can't write `{}`: {error}\n", target.display());
            EXIT_USAGE
        }
    }
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
    let binary = cpp.with_extension("checked");
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "g++".into());
    let built = std::process::Command::new(&compiler)
        .args(FLAGS)
        .arg("-o")
        .arg(&binary)
        .arg(cpp)
        .output();
    match built {
        Ok(output) if output.status.success() => {
            let _ = writeln!(
                stdout,
                "built {} ({compiler} {})",
                binary.display(),
                FLAGS.join(" ")
            );
            0
        }
        Ok(output) => {
            let errors = String::from_utf8_lossy(&output.stderr);
            let shown: Vec<&str> = errors.lines().take(20).collect();
            *stderr += &format!(
                "cpl: {compiler} could not build `{}`:\n{}\n",
                cpp.display(),
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
