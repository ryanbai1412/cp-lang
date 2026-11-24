//! Every command's output is pinned: each run below prints exactly the text
//! saved in `tests/pins/<command>.txt` (stdout, stderr and exit status), and
//! every command `cpl --help --json` lists has a pinned run of its text
//! output and, when it has JSON output, one of its JSON. A new command fails
//! here until it has both. After a change meant to move output, rerun with
//! `CPL_BLESS=1 cargo nextest run -p cpl-cli --test output_pins` and review
//! the diff.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[path = "support/cli_fixtures.rs"]
mod cli_fixtures;

const URL: &str = "https://atcoder.jp/contests/abc999/tasks/abc999_a";

/// Each run: its arguments and the protocol messages it reads on standard
/// input.
const RUNS: &[(&[&str], &[&str])] = &[
    (&["check", "a.cp", "bad.cp"], &[]),
    (&["check", "a.cp", "bad.cp", "--json"], &[]),
    (&["fix", "bad.cp"], &[]),
    (&["fix", "bad.cp", "--json"], &[]),
    (&["migrate", "a.cp"], &[]),
    (&["migrate", "a.cp", "--json"], &[]),
    (&["run", "a.cp", "a.tests/1.in"], &[]),
    (&["run", "a.cp", "a.tests/1.in", "--format", "json"], &[]),
    (&["run", "bad_index.cp", "--samples"], &[]),
    (
        &["run", "bad_index.cp", "--samples", "--format", "json"],
        &[],
    ),
    (&["judge", "j.cp", "j.in", "j.wrong"], &[]),
    (&["judge", "j.cp", "j.in", "j.right", "--json"], &[]),
    (&["build", "a.cp", "-o", "a.cpp"], &[]),
    (&["build", "a.cp", "-o", "a.cpp", "--json"], &[]),
    (&["vendor", "v.cp"], &[]),
    (&["vendor", "v.cp", "--json"], &[]),
    (&["judges"], &[]),
    (&["judges", "--json"], &[]),
    (&["submit", "a.cp"], &[]),
    (&["submit", "a.cp", "--json"], &[]),
    (&["fmt", "--check", "a.cp"], &[]),
    (&["fmt", "--check", "a.cp", "--json"], &[]),
    (&["lint", "a.cp"], &[]),
    (&["lint", "a.cp", "--json"], &[]),
    (&["explain", "E0301"], &[]),
    (&["explain", "a.cp", "--json"], &[]),
    (&["doc", "a.cp"], &[]),
    (&["doc", "--format", "json", "a.cp"], &[]),
    (&["new", "fresh"], &[]),
    (&["new", "fresh_json", "--json"], &[]),
    (&["fetch", URL, "got", "--html", "page.html"], &[]),
    (
        &["fetch", URL, "got_json", "--html", "page.html", "--json"],
        &[],
    ),
    (&["test", "a"], &[]),
    (&["test", "a", "--format", "json"], &[]),
    (&["stress", "differ.cp"], &[]),
    (&["stress", "same.cp", "--format", "json"], &[]),
    (&["locate", "alt.cp", "a.tests/1.in"], &[]),
    (&["locate", "alt.cp", "a.tests/1.in", "--json"], &[]),
    (&["bench", "a"], &[]),
    (&["bench", "a", "--format", "json"], &[]),
    (&["profile", "a.cp", "a.tests/1.in"], &[]),
    (
        &["profile", "a.cp", "a.tests/1.in", "--format", "json"],
        &[],
    ),
    (&["cover", "a.cp", "a.tests/1.in"], &[]),
    (&["cover", "a.cp", "a.tests/1.in", "--format", "json"], &[]),
    (&["watch", "a", "--once"], &[]),
    (&["watch", "a", "--once", "--json"], &[]),
    (&["report", "a.cp", "--input", "a.tests/1.in"], &[]),
    (
        &["report", "a.cp", "--input", "a.tests/1.in", "--json"],
        &[],
    ),
    (&["lsp"], LSP_SESSION),
    (&["dap"], DAP_SESSION),
    (&["replay", "r.cp", "--trace", "t.json", "--step", "1"], &[]),
    (
        &[
            "replay", "r.cp", "--trace", "t.json", "--step", "1", "--json",
        ],
        &[],
    ),
    (&["setup", "neovim", "--check"], &[]),
    (&["setup", "neovim", "--check", "--json"], &[]),
    (&["self", "list"], &[]),
    (&["self", "list", "--json"], &[]),
];

const LSP_SESSION: &[&str] = &[
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#,
    r#"{"jsonrpc":"2.0","method":"exit"}"#,
];

const DAP_SESSION: &[&str] = &[
    r#"{"seq":1,"type":"request","command":"initialize","arguments":{"adapterID":"cpl"}}"#,
    r#"{"seq":2,"type":"request","command":"disconnect"}"#,
];

/// Protocol messages as `Content-Length`-framed standard input.
fn framed(messages: &[&str]) -> String {
    messages
        .iter()
        .map(|body| format!("Content-Length: {}\r\n\r\n{body}", body.len()))
        .collect()
}

fn cpl(dir: &Path, home: &Path, args: &[&str], stdin: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .env("HOME", home)
        .env("CPL_HOME", home.join(".cpl"))
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cpl runs");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

/// What a run printed, the same on every machine: CRLF as LF, the fixture
/// folder as `$DIR` and its home inside as `~`, and each measured quantity as `<n>`: times in µs or
/// ns, times under a second in ms (limits are whole seconds), the JSON
/// `time_ms`, `time_ns` and `peak_kb`, a share or peak "of" a limit, and a
/// number right after a measured one (a profile's share, a bench row's
/// peak).
fn normalized(bytes: &[u8], dir: &Path) -> String {
    let text = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    let text = text.replace(&dir.join("home").display().to_string(), "~");
    let text = text.replace(&dir.display().to_string(), "$DIR");
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    let mut last_masked = false;
    while let Some(at) = rest.find(|c: char| c.is_ascii_digit()) {
        let (before, from) = rest.split_at(at);
        let len = from
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(from.len());
        let (number, after) = from.split_at(len);
        out.push_str(before);
        rest = after;
        if before.ends_with(|c: char| c.is_ascii_alphanumeric() || "_.-".contains(c)) {
            out.push_str(number);
            last_masked = false;
            continue;
        }
        let unit = after.trim_start_matches([' ', '\u{a0}']);
        let unit = unit
            .split(|c: char| !c.is_alphanumeric() && c != '%')
            .next()
            .unwrap_or("");
        let follows =
            last_masked && !before.contains('\n') && before.trim().chars().all(char::is_alphabetic);
        let measured = ["\"time_ms\":", "\"time_ns\":", "\"peak_kb\":"]
            .iter()
            .any(|key| before.ends_with(key))
            || matches!(unit, "us" | "µs" | "ns")
            || (unit == "ms" && number.parse::<f64>().is_ok_and(|n| n < 1000.0))
            || (after.starts_with("% of the ")
                && after[9..].starts_with(|c: char| c.is_ascii_digit()))
            || after.starts_with(" MB of ")
            || (follows && matches!(unit, "%" | "MB" | "KB"));
        out.push_str(if measured { "<n>" } else { number });
        last_masked = measured;
    }
    out.push_str(rest);
    machine_free(&out)
}

/// A bug report embeds the machine (OS, compiler, cpl's commit): its issue
/// URL's body, its size and its content-hashed name.
fn machine_free(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("cpl-report-") {
        let end = rest[at..].find(".md").map_or(rest.len(), |e| at + e);
        out.push_str(&rest[..at]);
        out.push_str("cpl-report-<hash>");
        rest = &rest[end..];
    }
    out.push_str(rest);
    let text = std::mem::take(&mut out);
    let mut rest = text.as_str();
    while let Some(at) = rest.find("&body=") {
        let end = rest[at..]
            .find(|c: char| c.is_whitespace() || c == '"')
            .map_or(rest.len(), |e| at + e);
        out.push_str(&rest[..at]);
        out.push_str("&body=<the report>");
        rest = &rest[end..];
    }
    out.push_str(rest);
    if out.contains("cpl-report-<hash>") {
        let text = std::mem::take(&mut out);
        let mut rest = text.as_str();
        while let Some(at) = rest.find(|c: char| c.is_ascii_digit()) {
            let len = rest[at..]
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len() - at);
            let before = &rest[..at];
            let after = &rest[at + len..];
            out.push_str(before);
            let size = (before.ends_with(".md (") && after.starts_with(" bytes"))
                || before.ends_with("\"bytes\":");
            out.push_str(if size { "<n>" } else { &rest[at..at + len] });
            rest = after;
        }
        out.push_str(rest);
    }
    out
}

/// The JSON flag a run uses, if any.
fn json_flag(args: &[&str]) -> Option<&'static str> {
    if args.contains(&"--json") {
        Some("--json")
    } else if args.windows(2).any(|w| w == ["--format", "json"]) {
        Some("--format json")
    } else {
        None
    }
}

#[test]
fn every_command_prints_its_pinned_output() {
    let pins = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/pins");
    let dir = cli_fixtures::fixtures("output-pins");
    let home = dir.join("home");
    let trace = cpl(
        &dir,
        &home,
        &["run", "r.cp", "--trace", "--format", "json"],
        "",
    );
    std::fs::write(dir.join("t.json"), &trace.stdout).unwrap();

    let mut errors = Vec::new();
    let help = cpl(&dir, &home, &["--help", "--json"], "");
    let help = cpl_json::parse(&String::from_utf8_lossy(&help.stdout)).unwrap();
    let mut commands = Vec::new();
    for command in help.at(&["commands"]).as_array().unwrap() {
        let name = command.at(&["name"]).as_str().unwrap();
        let json = command.at(&["json"]).as_str();
        let runs: Vec<_> = RUNS.iter().filter(|(args, _)| args[0] == name).collect();
        if !runs.iter().any(|(args, _)| json_flag(args).is_none()) {
            errors.push(format!("`cpl {name}` has no pinned run of its text output"));
        }
        if let Some(flag) = json {
            if !runs.iter().any(|(args, _)| json_flag(args) == Some(flag)) {
                errors.push(format!(
                    "`cpl {name}` has no pinned run of its `{flag}` output"
                ));
            }
        }
        commands.push(name.to_string());
    }
    for (args, _) in RUNS {
        if !commands.iter().any(|c| c == args[0]) {
            errors.push(format!(
                "`cpl {}` is not a command `cpl --help` lists",
                args.join(" ")
            ));
        }
    }

    let mut printed: BTreeMap<String, String> = BTreeMap::new();
    for (args, stdin) in RUNS {
        let out = cpl(&dir, &home, args, &framed(stdin));
        let shown = printed.entry(args[0].to_string()).or_default();
        shown.push_str(&format!(
            "$ cpl {}\nexit {}\n-- stdout\n{}-- stderr\n{}\n",
            args.join(" "),
            out.status.code().map_or("signal".into(), |c| c.to_string()),
            normalized(&out.stdout, &dir),
            normalized(&out.stderr, &dir),
        ));
    }

    let bless = cpl_env::BLESS.is_set();
    let mut differ = Vec::new();
    for (command, text) in &printed {
        let path = pins.join(format!("{command}.txt"));
        if std::fs::read_to_string(&path).ok().as_deref() == Some(text.as_str()) {
            continue;
        }
        if bless {
            std::fs::create_dir_all(&pins).unwrap();
            std::fs::write(&path, text).unwrap();
        } else {
            differ.push(format!("tests/pins/{command}.txt"));
        }
    }
    for entry in std::fs::read_dir(&pins).into_iter().flatten().flatten() {
        let path: PathBuf = entry.path();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        if !printed.contains_key(&stem) {
            if bless {
                std::fs::remove_file(&path).unwrap();
            } else {
                differ.push(format!("tests/pins/{stem}.txt pins no command's run"));
            }
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    assert!(
        differ.is_empty(),
        "output differs from the pins:\n{}\n\nIf the change is meant to move it, re-record with \
         `CPL_BLESS=1 cargo nextest run -p cpl-cli --test output_pins` and commit the pins with \
         the change.",
        differ.join("\n")
    );
}
