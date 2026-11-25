//! `cpl submit` and `cpl judges`: the file a judge receives, its size
//! against the limit, and the judge profiles.

use std::io::Write;

use cpl_diag::view::{Status, View};
use cpl_json::{obj, Value};

use crate::build::{build_file, BuildOptions};
use crate::{EXIT_ERRORS, EXIT_USAGE};
use cpl_diag::Shown;

/// `cpl submit`: `cpl build`, then `cpl test` on what it built; with
/// `json`, one object with `ready` and the `build` and `test` reports.
pub(crate) fn submit(
    path: &str,
    copy: bool,
    json: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let report =
        |ready: bool, build: &[u8], test: &[u8], said: &str, stdout: &mut (dyn Write + Send)| {
            if json {
                let parse =
                    |out: &[u8]| cpl_json::parse(&String::from_utf8_lossy(out)).unwrap_or_default();
                let (build, test) = (parse(build), parse(test));
                let report = cpl_json::write_with(|w| {
                    w.object().key("ready").bool(ready);
                    w.key("build").value(&build).key("test").value(&test);
                    view(ready, &build, &test, said).write(w);
                    w.end();
                });
                let _ = writeln!(stdout, "{report}");
            }
        };
    let mut built = Vec::new();
    let options = BuildOptions {
        json,
        checks: crate::build::JUDGE_CHECKS,
        ..BuildOptions::default()
    };
    let code = build_file(path, &options, &mut built, stderr);
    if !json {
        let _ = stdout.write_all(&built);
    }
    if code != 0 {
        *stderr += &format!("cpl: `{path}` is not ready to send: it does not build\n");
        report(false, &built, &[], stderr, stdout);
        return code;
    }
    let mut sink = Vec::new();
    let said: &mut (dyn Write + Send) = if json { &mut sink } else { stdout };
    warn_interpreted(path, "the cases", stderr);
    let source = cpl_contest::problem::source_path(path);
    let cases = cpl_contest::problem::cases(&source);
    let tests = cpl_contest::problem::find_tests_dir(&source);
    if cases.is_empty() {
        *stderr += &format!(
            "cpl: warning: {} has no cases, so only the build was checked; put the samples there as `1.in` and \
             `1.out`\n",
            tests.shown()
        );
        let code = ready_to_send(&source, copy, said, stderr);
        report(true, &built, &[], stderr, stdout);
        return code;
    }
    let mut out = Vec::new();
    let mut errors = Vec::new();
    let mut args = vec![path.to_string()];
    if json {
        args.extend(["--format".to_string(), "json".to_string()]);
    }
    let code = cpl_contest::test::run(&args, &mut out, &mut errors);
    if !json {
        let _ = said.write_all(&out);
    }
    *stderr += &String::from_utf8_lossy(&errors);
    if code != 0 {
        *stderr += &format!(
            "cpl: `{path}` is not ready to send: a case in {} fails\n",
            tests.shown()
        );
        report(false, &built, &out, stderr, stdout);
        return EXIT_ERRORS;
    }
    let code = ready_to_send(&source, copy, said, stderr);
    report(true, &built, &out, stderr, stdout);
    code
}

/// What `cpl submit --json` shows in an editor: one line saying whether the
/// file is ready, its size against the judge's limit and the samples it
/// passed, as the status and the panel, and under it what stopped it.
fn view(ready: bool, build: &Value, test: &Value, said: &str) -> View {
    let path = build.at(&["path"]).as_str().unwrap_or("the file");
    let size = build
        .at(&["bytes"])
        .as_u64()
        .map_or(String::new(), |bytes| {
            let share = build
                .at(&["limit"])
                .as_u64()
                .filter(|&limit| limit > 0)
                .map_or(String::new(), |limit| {
                    let judge = build.at(&["judge"]).as_str().unwrap_or("the judge");
                    let percent = (100.0 * bytes as f64 / limit as f64).round() as u64;
                    format!(", {percent}% of {judge}'s limit")
                });
            format!(" ({:.1} KB{share})", bytes as f64 / 1024.0)
        });
    let samples = match test.at(&["case_count"]).as_u64() {
        Some(total) => {
            let passed = test.at(&["passed_count"]).as_u64().unwrap_or(0);
            format!(", {passed}/{total} samples")
        }
        None => ", no samples".into(),
    };
    let verdict = if ready {
        "ready to send"
    } else {
        "not ready to send"
    };
    let copied = if ready && said.contains("cpl: copied ") {
        "; copied to the clipboard"
    } else {
        ""
    };
    let label = format!("{verdict}: {path}{size}{samples}{copied}");
    let mut panel = format!("{label}\n");
    if !ready {
        for line in said.trim_end().lines() {
            panel += &format!("  {line}\n");
        }
    }
    View {
        panel,
        status: Some(Status {
            line: 0,
            ok: ready,
            label,
            detail: None,
        }),
        ..View::default()
    }
}

/// Says that `what` ran in the interpreter because `path`'s judge compiler
/// is missing here: same answers, but not the C++ the judge runs.
pub(crate) fn warn_interpreted(path: &str, what: &str, stderr: &mut String) {
    if let Some(compiler) = cpl_stress::missing_compiler(std::path::Path::new(path)) {
        *stderr += &format!(
            "cpl: warning: there is no `{compiler}` here, so {what} ran in the interpreter, not the \
             judge's C++; install a C++ compiler to check that build\n"
        );
    }
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
                    cpp.shown()
                )
            }
            Err(error) => *stderr += &format!("cpl: warning: {error}; send the file instead\n"),
        }
    }
    let _ = writeln!(stdout, "ready to send: {}", cpp.shown());
    0
}

/// Environment variables set for one tool's run.
type Env = &'static [(&'static str, &'static str)];

/// A system clipboard tool `--copy` can pipe the C++ into.
#[derive(Debug, PartialEq)]
pub(crate) struct ClipTool {
    pub(crate) words: &'static [&'static str],
    /// Set for the run: pbcopy decodes its input by the locale, and under
    /// `C` turns UTF-8 into mojibake.
    pub(crate) env: Env,
}

const fn tool(words: &'static [&'static str]) -> ClipTool {
    ClipTool { words, env: &[] }
}

/// The clipboard tools to try on `os` (a `std::env::consts::OS`), in order;
/// the first that runs and succeeds wins.
pub(crate) fn clipboard_tools(os: &str) -> &'static [ClipTool] {
    const MAC: &[ClipTool] = &[ClipTool {
        env: &[("LC_ALL", "en_US.UTF-8")],
        ..tool(&["pbcopy"])
    }];
    const WINDOWS: &[ClipTool] = &[tool(&["clip"])];
    // Wayland, X11, then WSL's bridge to the Windows clipboard.
    const UNIX: &[ClipTool] = &[
        tool(&["wl-copy"]),
        tool(&["xclip", "-selection", "clipboard"]),
        tool(&["xsel", "--clipboard", "--input"]),
        tool(&["clip.exe"]),
    ];
    match os {
        "macos" => MAC,
        "windows" => WINDOWS,
        _ => UNIX,
    }
}

/// Puts `text` on the clipboard with the first tool this machine has, or
/// the command in `CPL_CLIPBOARD`. Returns the tool's name.
pub(crate) fn copy_to_clipboard(text: &str) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let chosen = cpl_env::CLIPBOARD.text();
    let tools = clipboard_tools(std::env::consts::OS);
    let candidates: Vec<(Vec<String>, Env)> = match &chosen {
        Some(command) => vec![(vec!["sh".into(), "-c".into(), command.clone()], &[])],
        None => tools
            .iter()
            .map(|tool| (tool.words.iter().map(|w| w.to_string()).collect(), tool.env))
            .collect(),
    };
    for (words, env) in &candidates {
        let Ok(mut child) = Command::new(&words[0])
            .args(&words[1..])
            .envs(env.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            continue;
        };
        if let Some(mut pipe) = child.stdin.take() {
            let _ = pipe.write_all(text.as_bytes());
        }
        if child.wait().is_ok_and(|status| status.success()) {
            return Ok(chosen.clone().unwrap_or_else(|| words.join(" ")));
        }
    }
    Err(match chosen {
        Some(command) => format!("`CPL_CLIPBOARD` (`{command}`) failed"),
        None => {
            let names: Vec<&str> = tools.iter().map(|tool| tool.words[0]).collect();
            format!(
                "no clipboard tool here ({}); set `CPL_CLIPBOARD` to one",
                names.join(", ")
            )
        }
    })
}

/// Where the bytes of a built file go, by part, with the largest pieces of
/// each part named.
pub(crate) fn size_report(sizes: &[cpl_facade::Size], limit: usize, judge: &str) -> String {
    let total: usize = sizes.iter().map(|size| size.bytes).sum();
    let mut out =
        format!("size of the C++ by part: {total} bytes, of the {limit} of judge `{judge}`\n");
    let mut parts: Vec<cpl_facade::Part> = sizes.iter().map(|size| size.part).collect();
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
            return EXIT_ERRORS;
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
                    "thread_stack_kb",
                    profile.thread_stack.map_or(Value::Null, |b| int(b >> 10)),
                ),
                ("source_limit_kb", int(profile.limit() as u64 / 1024)),
                ("time_limit_ms", int(profile.time_ms)),
                ("memory_limit_kb", int(profile.memory_mb * 1024)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_os_gets_its_own_clipboard_tools() {
        let names = |os| -> Vec<String> {
            clipboard_tools(os)
                .iter()
                .map(|tool| tool.words.join(" "))
                .collect()
        };
        assert_eq!(names("macos"), ["pbcopy"]);
        assert_eq!(names("windows"), ["clip"]);
        let unix = [
            "wl-copy",
            "xclip -selection clipboard",
            "xsel --clipboard --input",
            "clip.exe",
        ];
        assert_eq!(names("linux"), unix);
        assert_eq!(names("freebsd"), unix);
        assert_eq!(clipboard_tools("macos")[0].env, [("LC_ALL", "en_US.UTF-8")]);
    }

    fn label(ready: bool, build: &str, test: &str, said: &str) -> (String, String) {
        let parse = |text: &str| cpl_json::parse(text).unwrap();
        let view = view(ready, &parse(build), &parse(test), said);
        assert_eq!(view.status.as_ref().unwrap().ok, ready);
        (view.status.unwrap().label, view.panel)
    }

    #[test]
    fn the_view_says_if_it_is_ready_how_big_and_how_many_samples_pass() {
        let build = r#"{"bytes":4593,"judge":"codeforces","limit":65536,"path":"a.cpp"}"#;
        let test = r#"{"cases":[],"passed_count":2,"case_count":2}"#;
        let (status, panel) = label(true, build, test, "");
        assert_eq!(
            status,
            "ready to send: a.cpp (4.5 KB, 7% of codeforces's limit), 2/2 samples"
        );
        assert_eq!(panel, format!("{status}\n"));
        let copied = "cpl: copied a.cpp to the clipboard with `xclip`\n";
        assert!(label(true, build, test, copied)
            .0
            .ends_with("2/2 samples; copied to the clipboard"));
        let (status, panel) = label(
            false,
            r#"{"path":"a.cpp","bytes":10}"#,
            r#"{"passed_count":0,"case_count":1}"#,
            "cpl: `a.cp` is not ready to send: a case in a.tests fails\n",
        );
        assert_eq!(status, "not ready to send: a.cpp (0.0 KB), 0/1 samples");
        assert_eq!(
            panel,
            format!("{status}\n  cpl: `a.cp` is not ready to send: a case in a.tests fails\n")
        );
        // A build that failed has no report; a problem with no cases, no test.
        assert_eq!(
            label(false, "null", "null", "").0,
            "not ready to send: the file, no samples"
        );
        let unlimited = r#"{"path":"a.cpp","bytes":2048,"limit":0}"#;
        assert_eq!(
            label(true, unlimited, "null", "").0,
            "ready to send: a.cpp (2.0 KB), no samples"
        );
        let unnamed = r#"{"path":"a.cpp","bytes":32768,"limit":65536}"#;
        assert!(label(true, unnamed, "null", "")
            .0
            .contains("50% of the judge's limit"));
    }
}
