//! `cpl submit` and `cpl judges`: the file a judge receives, its size
//! against the limit, and the judge profiles.

use std::io::Write;

use cpl_json::{obj, Value};

use crate::{build_file, EXIT_ERRORS, EXIT_USAGE};

/// `cpl submit`: `cpl build`, then `cpl test` on what it built.
pub(crate) fn submit(
    path: &str,
    copy: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let code = build_file(path, None, None, false, false, false, stdout, stderr);
    if code != 0 {
        *stderr += &format!("cpl: `{path}` is not ready to send: it does not build\n");
        return code;
    }
    let source = cpl_contest::problem::source_path(path);
    let cases = cpl_contest::problem::cases(&source);
    let tests = cpl_contest::problem::find_tests_dir(&source);
    if cases.is_empty() {
        *stderr += &format!(
            "cpl: warning: {} has no cases, so only the build was checked; put the samples there as `1.in` and \
             `1.out`\n",
            tests.display()
        );
        return ready_to_send(&source, copy, stdout, stderr);
    }
    let mut out = Vec::new();
    let mut errors = Vec::new();
    let code = cpl_contest::test::run(&[path.to_string()], &mut out, &mut errors);
    let _ = stdout.write_all(&out);
    *stderr += &String::from_utf8_lossy(&errors);
    if code != 0 {
        *stderr += &format!(
            "cpl: `{path}` is not ready to send: a case in {} fails\n",
            tests.display()
        );
        return EXIT_ERRORS;
    }
    ready_to_send(&source, copy, stdout, stderr)
}

pub(crate) fn ready_to_send(
    source: &std::path::Path,
    copy: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let cpp = source.with_extension("cpp");
    if copy {
        match std::fs::read_to_string(&cpp)
            .map_err(|e| e.to_string())
            .and_then(|text| copy_to_clipboard(&text))
        {
            Ok(tool) => {
                *stderr += &format!(
                    "cpl: copied {} to the clipboard with `{tool}`\n",
                    cpp.display()
                )
            }
            Err(error) => *stderr += &format!("cpl: warning: {error}; send the file instead\n"),
        }
    }
    let _ = writeln!(stdout, "ready to send: {}", cpp.display());
    0
}

/// Puts `text` on the clipboard with the first tool this machine has, or
/// the command in `CPL_CLIPBOARD`. Returns the tool's name.
pub(crate) fn copy_to_clipboard(text: &str) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let chosen = std::env::var("CPL_CLIPBOARD")
        .ok()
        .filter(|c| !c.trim().is_empty());
    let candidates: Vec<Vec<String>> = match &chosen {
        Some(command) => vec![vec!["sh".into(), "-c".into(), command.clone()]],
        None => [
            &["pbcopy"][..],
            &["wl-copy"],
            &["xclip", "-selection", "clipboard"],
            &["xsel", "--clipboard", "--input"],
            &["clip.exe"],
            &["clip"],
        ]
        .iter()
        .map(|words| words.iter().map(|w| w.to_string()).collect())
        .collect(),
    };
    for words in &candidates {
        let Ok(mut child) = Command::new(&words[0])
            .args(&words[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            continue;
        };
        if let Some(mut input) = child.stdin.take() {
            let _ = input.write_all(text.as_bytes());
        }
        if child.wait().is_ok_and(|status| status.success()) {
            return Ok(chosen.clone().unwrap_or_else(|| words.join(" ")));
        }
    }
    Err(match chosen {
        Some(command) => format!("`CPL_CLIPBOARD` (`{command}`) failed"),
        None => "no clipboard tool here (pbcopy, wl-copy, xclip, xsel or clip); set `CPL_CLIPBOARD` to one".into(),
    })
}

/// Where the bytes of a built file go, by part, with the largest pieces of
/// each part named.
pub(crate) fn size_report(sizes: &[cpl_codegen::Size], limit: usize, judge: &str) -> String {
    let total: usize = sizes.iter().map(|size| size.bytes).sum();
    let mut out =
        format!("size of the C++ by part: {total} bytes, of the {limit} of judge `{judge}`\n");
    let mut parts: Vec<cpl_codegen::Part> = sizes.iter().map(|size| size.part).collect();
    parts.sort();
    parts.dedup();
    for part in parts {
        let mut pieces: Vec<(&str, usize)> = Vec::new();
        for size in sizes.iter().filter(|size| size.part == part) {
            match pieces.iter_mut().find(|(name, _)| *name == size.name) {
                Some((_, bytes)) => *bytes += size.bytes,
                None => pieces.push((&size.name, size.bytes)),
            }
        }
        let bytes: usize = pieces.iter().map(|(_, bytes)| bytes).sum();
        if bytes == 0 {
            continue;
        }
        pieces.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        let mut named: Vec<String> = pieces
            .iter()
            .take(3)
            .map(|(name, bytes)| format!("{name} {bytes}"))
            .collect();
        if pieces.len() > 3 {
            named.push(format!("{} more", pieces.len() - 3));
        }
        let named = if pieces.len() > 1 {
            named.join(", ")
        } else {
            String::new()
        };
        let share = (bytes * 100).div_ceil(total.max(1));
        out += format!("  {:<12}{bytes:>7} {share:>3}%  {named}", part.describe()).trim_end();
        out.push('\n');
    }
    out
}

/// `cpl judges`: every judge profile, and with a file, the one its build is for.
pub(crate) fn list_judges(
    file: Option<&str>,
    json: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let path = std::path::Path::new(file.unwrap_or("./a.cp"));
    let source = match file {
        Some(file) => match std::fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{file}`: {error}\n");
                return EXIT_USAGE;
            }
        },
        None => String::new(),
    };
    let chosen = match cpl_judge::resolve(path, &source, None) {
        Ok(profile) => profile,
        Err(error) => {
            *stderr += &format!("cpl: {error}\n");
            return EXIT_USAGE;
        }
    };
    let config = cpl_judge::project_config(path).unwrap_or_default();
    let mut profiles = Vec::new();
    for name in config.names() {
        let Some(profile) = config.profile(&name) else {
            continue;
        };
        if json {
            let text = |s: &str| Value::String(s.to_string());
            let int = |n: u64| Value::Int(n as i64);
            profiles.push(obj([
                ("name", text(&profile.name)),
                ("title", text(&profile.title)),
                ("chosen", Value::Bool(profile.name == chosen.name)),
                ("command", text(&profile.command())),
                (
                    "os",
                    text(if profile.os == cpl_judge::Os::Windows {
                        "windows"
                    } else {
                        "linux"
                    }),
                ),
                ("stack", text(&profile.stack_text())),
                (
                    "thread_stack_mb",
                    profile
                        .thread_stack
                        .map_or(Value::Null, |b| int(b as u64 >> 20)),
                ),
                ("source_limit_kb", int(profile.limit() as u64 / 1024)),
                ("time_ms", int(profile.time_ms as u64)),
                ("memory_mb", int(profile.memory_mb as u64)),
                ("evidence", text(&profile.evidence)),
            ]));
            continue;
        }
        let mark = if profile.name == chosen.name {
            "*"
        } else {
            " "
        };
        let _ = writeln!(
            stdout,
            "{mark} {:<11}{}\n    {}\n    {}, {}, {}, source up to {} KB\n    usual limits {} ms, {} MB\n    from: {}",
            profile.name,
            profile.title,
            profile.command(),
            if profile.os == cpl_judge::Os::Windows { "Windows" } else { "Linux" },
            profile.stack_text(),
            match profile.thread_stack {
                Some(bytes) => format!("the program on a thread with {} MB", bytes >> 20),
                None => "no threads".into(),
            },
            profile.limit() / 1024,
            profile.time_ms,
            profile.memory_mb,
            profile.evidence
        );
    }
    if json {
        let judges = obj([("judges", Value::Array(profiles))]);
        let _ = writeln!(stdout, "{}", cpl_json::to_string(&judges));
    }
    0
}
