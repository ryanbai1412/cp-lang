//! What `cpl --help` and each command's `--help` print, and which usage a
//! command's usage error shows.

use std::sync::OnceLock;

use cpl_json::{obj, Value};

/// One `cpl` command: what `cpl --help` lists and `cpl --help --json` emits.
pub(crate) struct Command {
    pub name: &'static str,
    pub args: &'static str,
    /// The summary as `cpl --help` wraps it.
    pub summary: &'static [&'static str],
    /// How the command spells JSON output.
    pub json: Json,
    /// Every option, for the commands whose arguments `cpl` parses itself;
    /// `None` where the command's crate parses its own.
    pub options: Option<&'static [&'static str]>,
}

#[derive(PartialEq)]
pub(crate) enum Json {
    None,
    /// `--json`
    Flag,
    /// `--format json`
    Format,
}

pub(crate) const COMMANDS: &[Command] = &[
    Command {
        name: "check",
        args: "<file>...",
        summary: &["report every error in the files"],
        json: Json::Flag,
        options: Some(&["--json"]),
    },
    Command {
        name: "fix",
        args: "<file>...",
        summary: &["apply each error's suggested fix, report what is left"],
        json: Json::Flag,
        options: Some(&["--json"]),
    },
    Command {
        name: "migrate",
        args: "[path]...",
        summary: &["rewrite removed spellings to the current ones"],
        json: Json::Flag,
        options: Some(&["--check", "--json"]),
    },
    Command {
        name: "run",
        args: "<file> [input]",
        summary: &[
            "check, then run in the debug interpreter, reading",
            "the input file or standard input; a file with an",
            "`@interactor` runs against it, on the hidden test",
            "(`cpl run --help` has the options for traces);",
            "`--release` runs the C++ that `cpl build` writes",
        ],
        json: Json::Format,
        options: None,
    },
    Command {
        name: "judge",
        args: "<file> <input> [<output> <answer>]",
        summary: &[
            "run the file's `@interactor` as a judge program on",
            "standard input and output, or with an output its",
            "`@check`; exit status 0 accepts",
        ],
        json: Json::Flag,
        options: Some(&["--json"]),
    },
    Command {
        name: "build",
        args: "<file> [-o out]",
        summary: &[
            "check, then write one C++17 file to submit: next to",
            "the source by default, to standard output with `-o -`;",
            "`--checked` also builds it with sanitizers to run here;",
            "`--judge <name>` writes it for that judge",
        ],
        json: Json::Flag,
        options: Some(&[
            "-o",
            "--judge",
            "--checks",
            "--checked",
            "--compiler",
            "--size",
            "--copy",
            "--json",
        ]),
    },
    Command {
        name: "vendor",
        args: "<file>",
        summary: &["copy the std modules the file loads to vendor/std/"],
        json: Json::None,
        options: None,
    },
    Command {
        name: "judges",
        args: "[file]",
        summary: &["list the judge profiles, marking the one a build uses"],
        json: Json::Flag,
        options: Some(&["--json"]),
    },
    Command {
        name: "submit",
        args: "<file>",
        summary: &[
            "build for the file's judge, run its cases on that",
            "build, and say whether it is ready to send",
        ],
        json: Json::Flag,
        options: Some(&["--copy", "--json"]),
    },
    Command {
        name: "fmt",
        args: "[path]...",
        summary: &["format files in the one canonical style"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "lint",
        args: "[path]...",
        summary: &["warn about code that is legal but probably wrong or slow"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "explain",
        args: "<CODE|file|--json>",
        summary: &[
            "show a diagnostic or lint explanation, or a file's",
            "representations and view costs",
        ],
        json: Json::Flag,
        options: Some(&["--json"]),
    },
    Command {
        name: "doc",
        args: "[path]...",
        summary: &["build HTML documentation from `///` comments"],
        json: Json::Format,
        options: None,
    },
    Command {
        name: "new",
        args: "<name>",
        summary: &["create a problem from its constraints or samples"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "fetch",
        args: "<url> [name]",
        summary: &["download a Codeforces, AtCoder or CSES problem's samples"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "test",
        args: "<problem>",
        summary: &["run every case in <problem>.tests/"],
        json: Json::Format,
        options: None,
    },
    Command {
        name: "stress",
        args: "<file>",
        summary: &["run against a brute on random inputs; shrink the failure"],
        json: Json::Format,
        options: None,
    },
    Command {
        name: "locate",
        args: "<file> [input]",
        summary: &["the first part whose `@alt` disagrees on an input"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "bench",
        args: "<problem>",
        summary: &["run on maximum-size worst-case inputs against the limits"],
        json: Json::Format,
        options: None,
    },
    Command {
        name: "profile",
        args: "<file> [input]",
        summary: &["run, then show each line's hits and share of the time"],
        json: Json::Format,
        options: Some(&["--native", "--format"]),
    },
    Command {
        name: "cover",
        args: "<file> [input]...",
        summary: &["list the lines and branches no input reached"],
        json: Json::Format,
        options: Some(&["--native", "--format"]),
    },
    Command {
        name: "watch",
        args: "<problem>",
        summary: &["check, lint and test on every save"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "report",
        args: "<file>",
        summary: &["write a paste-ready bug report (`--last` after an ICE)"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "lsp",
        args: "",
        summary: &["serve the language server protocol on stdio"],
        json: Json::None,
        options: Some(&["--stdio"]),
    },
    Command {
        name: "dap",
        args: "",
        summary: &["serve the time-travel debug adapter (DAP) on stdio"],
        json: Json::None,
        options: Some(&["--stdio"]),
    },
    Command {
        name: "replay",
        args: "<file> --trace <t.json> --step <n>",
        summary: &["run one recorded call again with the file's edited fn"],
        json: Json::Flag,
        options: None,
    },
    Command {
        name: "setup",
        args: "<neovim | vim | vscode | jetbrains>",
        summary: &["install the editor support: `cpl setup neovim`"],
        json: Json::Flag,
        options: Some(&["--dir", "--vsix", "--zip", "--check", "--json"]),
    },
    Command {
        name: "self",
        args: "<command>",
        summary: &[
            "update, switch or roll back the installed toolchain",
            "(`cpl self --help`); `cpl +<toolchain> <command>` runs",
            "one command with another, e.g. `cpl +edge run a.cp`",
        ],
        json: Json::Flag,
        options: None,
    },
];

const HEADER: &str = "\
usage: cpl <command> [args]

commands:
";

const FOOTER: &str = "
`cpl <command> --help` describes a command's options.
`cpl b`, `cpl r` and `cpl t` are `build`, `run` and `test`; `-c` is `--copy`.

options:
  -V, --version    print the version
  -h, --help       print this help
";

/// `cpl --help --json`: every command with its usage, summary and own
/// `--help`, the contract the site's CLI reference is generated from.
pub(crate) fn help_json() -> String {
    let text = |s: String| Value::String(s);
    let commands = COMMANDS
        .iter()
        .map(|c| {
            let help = crate::run(&[c.name, "--help"]);
            let help = String::from_utf8_lossy(&help.stdout).trim_end().to_string();
            obj([
                ("name", text(c.name.into())),
                (
                    "usage",
                    text(format!("{} {}", c.name, c.args).trim_end().into()),
                ),
                ("summary", text(c.summary.join(" "))),
                (
                    "json",
                    match c.json {
                        Json::Flag => text("--json".into()),
                        Json::Format => text("--format json".into()),
                        Json::None => Value::Null,
                    },
                ),
                (
                    "help",
                    if help.is_empty() {
                        Value::Null
                    } else {
                        text(help)
                    },
                ),
            ])
        })
        .collect();
    let version = text(crate::VERSION.into());
    cpl_json::to_string(&obj([
        ("version", version),
        ("commands", Value::Array(commands)),
    ]))
}

/// What `cpl --help` prints: [`COMMANDS`] between [`HEADER`] and [`FOOTER`].
pub(crate) fn usage() -> &'static str {
    static USAGE: OnceLock<String> = OnceLock::new();
    USAGE.get_or_init(|| {
        let mut out = HEADER.to_string();
        for command in COMMANDS {
            let left = format!("{} {}", command.name, command.args);
            let left = left.trim_end();
            let mut summary = command.summary.iter();
            match summary.next() {
                Some(first) if left.len() < 22 => out += &format!("  {left:<22}{first}\n"),
                Some(first) => out += &format!("  {left}\n{:24}{first}\n", ""),
                None => out += &format!("  {left}\n"),
            }
            for line in summary {
                out += &format!("{:24}{line}\n", "");
            }
        }
        out + FOOTER
    })
}
/// Runs `cpl` with `args`, writing to `stdout` as it goes. `stdin` is
/// called only by a command that reads standard input: each call gives the
/// bytes that have arrived, and nothing once the input has ended.
/// A command's usage, for its `--help` and its usage errors.
pub(crate) fn command_usage(command: &str) -> Option<&'static str> {
    Some(match command {
        "check" => CHECK_USAGE,
        "fix" => FIX_USAGE,
        "migrate" => crate::migrate::USAGE,
        "setup" => crate::setup::USAGE,
        "run" => RUN_USAGE,
        "build" => BUILD_USAGE,
        "locate" => cpl_stress::locate::USAGE,
        "judge" => JUDGE_USAGE,
        "submit" => SUBMIT_USAGE,
        "judges" => JUDGES_USAGE,
        "vendor" => crate::vendor::USAGE,
        "explain" => EXPLAIN_USAGE,
        "lsp" => LSP_USAGE,
        "dap" => DAP_USAGE,
        "profile" => crate::profile::PROFILE_USAGE,
        "cover" => crate::profile::COVER_USAGE,
        _ => return None,
    })
}

const CHECK_USAGE: &str = "\
usage: cpl check <file.cp>... [--json]

Reports every error in the files: names, types, lending, input ranges.
Nothing is run and nothing is written. The exit status is 1 when a file
has an error.

  --json    print one JSON object instead: `files`, each with its `path`,
            `ok`, and `diagnostics` (code, severity, message, start, end,
            labels, notes, help, fixes; positions are 1-based line and byte
            column, with the byte offset)
";

const FIX_USAGE: &str = "\
usage: cpl fix <file.cp>... [--json]

Applies the fix each error carries (the quick fixes an editor offers),
then, once no error is left, the fixes `cpl lint --fix` makes. Rewrites the
file and reports the errors left. The exit status is 1 when an error is left.

  --json    print one JSON object instead: `files`, each with its `path`,
            the number of fixes `applied`, `ok`, and the `diagnostics` left,
            as `cpl check --json` prints them, in the rewritten file
";

const BUILD_USAGE: &str = "\
usage: cpl build <file.cp> [options]

Checks the file, then writes one C++17 file to submit, next to the source.

options:
  -o <out>          write to <out> instead; `-o -` writes to standard output
  --judge <name>    for that judge; else the file's `// judge:` line, else
                    `judge` under `[judge]` in the nearest cp.toml, else
                    `generic`. `cpl judges` lists them
  --checks <level>  what the program still checks on the judge: `none`
                    (the default) checks what hand-written C++ checks, with
                    `/ 0` giving 0; `traps` stops it on a bad index, `/ 0`
                    or a false `assert` (the default with `--checked`);
                    `all` adds range stores, overflow and the rest of the
                    local build's checks
  --checked         also compile it with sanitizers into a program to run here
  --compiler <toolchain>
                    build with that toolchain (`0.3.0`, `edge-2026-11-22`),
                    as `cpl +<toolchain> build` does, so an old solution
                    rebuilds byte for byte
  --size            say where the bytes go: header, runtime, library fns,
                    your code, by part and by piece (said anyway when the
                    file is over the judge's limit)
  -c, --copy        also put the C++ on the clipboard (pbcopy, wl-copy, xclip,
                    xsel or clip, else the command in CPL_CLIPBOARD)
  --json            print one JSON object once it is written: its `path`,
                    `bytes`, the `limit` and `judge`, and `sizes`, each
                    piece's `part`, `name` and `bytes`, as `--size` says them

Over 32 KB it warns; over 64 KB, or the judge's own smaller limit, it fails
and says where the bytes go. The same file and judge give the same bytes.
";

const JUDGES_USAGE: &str = "\
usage: cpl judges [file.cp] [--json]

Lists the judge profiles: compiler, flags, stack and source limit. With a
file, marks the one its build uses (the `// judge:` line, else the nearest
cp.toml, else `generic`).

  --json    print `{\"judges\": [...]}`, each profile with its name, title,
            chosen, command, os, stack, thread_stack_kb, source_limit_kb,
            time_limit_ms, memory_limit_kb and evidence
";

pub(crate) const EXPLAIN_USAGE: &str = "\
usage: cpl explain <CODE | file.cp | --json>

With a code such as E0302, explains that diagnostic or lint: what triggers
it and how to fix it. With a file, shows how each value is represented and
what its views cost. `--json` prints every explanation as JSON.
";

const SUBMIT_USAGE: &str = "\
usage: cpl submit <file.cp> [options]

Gets the file ready to send to its judge (the `// judge:` line, else the
nearest cp.toml, else `generic`): writes the C++ as `cpl build` does, within
the judge's source limit, then runs every case in <file>.tests/ on that C++
built with the judge's flags on its stack, as `cpl test` does. Exits 0 when
the file is ready: it built and no case failed.

options:
  -c, --copy        when ready, also put the C++ on the clipboard, as
                    `cpl build --copy` does
  --json            print one JSON object: `ready`, and the `build` and
                    `test` reports `cpl build --json` and `cpl test --json`
                    print (`null` when that step did not run)

Mid-contest: put `// judge: codeforces` (or another of `cpl judges`) at the
top of the file once, save the samples as <file>.tests/1.in and 1.out, then
`cpl submit a.cp --copy` and paste, or upload a.cpp.
";

const JUDGE_USAGE: &str = "\
usage: cpl judge <file.cp> <input> [--json]
       cpl judge <file.cp> <input> <output> [answer] [--json]

With an input alone, runs the file's `@interactor` on it as a judge
program: it talks to a solution on standard input and output. With an
output, runs the file's `@check` on that output. The exit status is 0
when the judge accepts, 3 when it rejects.

  --json    print one JSON object: `accepted`, and when it rejects, the
            `reason` and the `diagnostic` where the judge failed, as
            `cpl check --json` prints one
";

const LSP_USAGE: &str = "\
usage: cpl lsp

Serves the language server protocol on standard input and output, for an
editor to start: errors as you type, hover, go to definition.
";

const DAP_USAGE: &str = "\
usage: cpl dap

Serves the Debug Adapter Protocol (DAP) on standard input and output,
supporting breakpoints, stepping, backward time-travel (`stepBack`,
`reverseContinue`), call stacks, domain-labeled variables, and traps.
";

const RUN_USAGE: &str = "\
usage: cpl run <file.cp> [input | sample] [options]

Checks the file, then runs it in the debug interpreter on the input file
or on standard input. What `debug(...)` shows goes to standard error.
With no input named and none piped in, it runs every sample in
<file>.tests/ as `cpl test` does; a sample's name (`cpl run a 2`) runs
that one.

options:
  --trace [SPEC]    record the run: every change to a top-level variable,
                    every print and every debug(...); with SPEC, comma-
                    separated fn names or globs (`*`, `?`) with `!pat`
                    exclusions (e.g. `solve,!helper`). Writes
                    <file>.trace.html unless --format says where it goes.
                    With no input named and none piped in, it records the
                    case `cpl stress` saved last for the file
  --test SPEC       with --trace: record only these 1-based multitest cases
                    (e.g. `3`, `1,3`, `2..=4`)
  --watch VAR       with --trace: record steps only when VAR changes (repeatable
                    or comma-separated, e.g. `--watch dp,ans`)
  --last N          with --trace: keep only the last N steps (ring buffer)
  --from N          with --trace: skip the first N steps (0-based), to keep
                    any part of a long run
  --depth N         with --trace: record only up to call depth N (0 = top level)
  --full            with --trace: trace the whole run as it goes; by default
                    only the steps kept are traced and the rest runs untraced
  --format json     without --trace: the value of each top-level variable,
                    the output and the time, for an editor.
                    with --trace: the trace as JSON on standard output
  --format html     with --trace: the trace viewer with the trace in it,
                    one self-contained page on standard output
  --live            with --trace: serve the viewer on localhost and record
                    again each time the file is saved, on the same input;
                    with no input named, the case `cpl stress` saved last.
                    The page keeps its place and shows the first change
  --brute           run the file's `@alt(program)` fn in place of the solution
  --release         run what a judge runs: the C++ of `cpl build`, built with
                    the judge's flags (those g++ here accepts) on the judge's
                    stack, and kept until the file changes.
                    Fast, and without the interpreter's checks; it traps on
                    a bad index, `/ 0` and `assert` unless `--checks` says
  --checks LEVEL    with --release: what that build traps on, as
                    `cpl build --checks` takes it (none, traps, all)
  --time-limit MS   stop the run after MS milliseconds
";

/// Both spellings of JSON output, `--json` and `--format json`, work on every
/// command that has one (decisions.md 473).
/// Short forms typed mid-contest: `cpl b -c` is `cpl build --copy`.
const SHORT_COMMANDS: &[(&str, &str)] = &[("b", "build"), ("r", "run"), ("t", "test")];
const SHORT_OPTIONS: &[(&str, &str, &str)] =
    &[("build", "-c", "--copy"), ("submit", "-c", "--copy")];

/// Commands other contest tools spell differently, by what a contestant
/// would type.
const OTHER_SPELLINGS: &[(&str, &str)] = &[
    ("listen", "new --listen"),
    ("import", "new --listen"),
    ("parse", "new --listen"),
    ("init", "new"),
    ("compile", "build"),
    ("format", "fmt"),
    ("check-all", "test"),
    ("debug", "dap"),
    ("brute", "stress"),
    ("install", "setup"),
    ("update", "self update"),
    ("upgrade", "self update"),
];

/// The command an unknown `name` most likely meant: another tool's spelling,
/// else a command at most two edits away.
pub(crate) fn guess(name: &str) -> Option<String> {
    if let Some((_, command)) = OTHER_SPELLINGS.iter().find(|(other, _)| *other == name) {
        return Some(command.to_string());
    }
    COMMANDS
        .iter()
        .map(|c| c.name)
        .chain(SHORT_COMMANDS.iter().map(|(_, long)| *long))
        .map(|command| (cpl_diag::edit_distance(name, command), command))
        .filter(|(distance, command)| *distance <= 2.min(command.len() / 2))
        .min()
        .map(|(_, command)| command.to_string())
}

/// `args` with each short command and option replaced by its long form.
pub(crate) fn long_forms<S: AsRef<str>>(args: &[S]) -> Vec<String> {
    let mut long: Vec<String> = args.iter().map(|arg| arg.as_ref().to_string()).collect();
    if let Some(first) = long.first_mut() {
        if let Some((_, name)) = SHORT_COMMANDS.iter().find(|(short, _)| short == first) {
            *first = name.to_string();
        }
    }
    let command = long.first().cloned().unwrap_or_default();
    for arg in long.iter_mut().skip(1) {
        let short = SHORT_OPTIONS
            .iter()
            .find(|(c, short, _)| *c == command && short == arg);
        if let Some((_, _, name)) = short {
            *arg = name.to_string();
        }
    }
    long
}

pub(crate) fn json_spelling<'a>(command: &str, args: Vec<&'a str>) -> Vec<&'a str> {
    let json = COMMANDS.iter().find(|c| c.name == command).map(|c| &c.json);
    if json == Some(&Json::Format) {
        let spell = |arg| match arg {
            "--json" => vec!["--format", "json"],
            arg => vec![arg],
        };
        return args.into_iter().flat_map(spell).collect();
    }
    if json != Some(&Json::Flag) {
        return args;
    }
    let mut spelled = Vec::new();
    let mut args = args.into_iter().peekable();
    while let Some(arg) = args.next() {
        if arg == "--format" && args.peek() == Some(&"json") {
            args.next();
            spelled.push("--json");
        } else {
            spelled.push(arg);
        }
    }
    spelled
}

/// `cpl run a` for `cpl run a.cp`, as `cpl test a` already reads it: the
/// file argument of a command that takes one gets `.cp` when it has no
/// extension, names nothing, and `<it>.cp` is a file. Only the first
/// argument, or every non-option one for `<file>...` commands, is a file.
pub(crate) fn file_names<'a>(command: &str, args: Vec<&'a str>) -> Vec<std::borrow::Cow<'a, str>> {
    use std::borrow::Cow;
    let shape = COMMANDS
        .iter()
        .find(|c| c.name == command)
        .map_or("", |c| c.args);
    let many = shape == "<file>...";
    let resolve = |arg: &'a str| -> Cow<'a, str> {
        match bare_source(std::path::Path::new(arg)) {
            Some(cp) => Cow::Owned(cp.display().to_string()),
            None => Cow::Borrowed(arg),
        }
    };
    let takes_file = shape.starts_with("<file") || shape == "<problem>";
    let names_file = |arg: &&str| {
        let path = std::path::Path::new(arg);
        !arg.starts_with('-')
            && (arg.ends_with(".cp") || path.exists() || path.with_extension("cp").is_file())
    };
    // `--last` (`cpl report`) names its input another way.
    let needs_none = args
        .iter()
        .any(|arg| matches!(*arg, "-h" | "--help" | "--last"));
    let newest = match takes_file && !needs_none && !args.iter().any(names_file) {
        true => newest_source(std::path::Path::new(".")),
        false => None,
    };
    let named: Vec<Cow<'a, str>> = args
        .into_iter()
        .enumerate()
        .map(
            // A `<problem>` stays as written unless a binary shadows its name.
            |(i, arg)| match takes_file
                && (shape != "<problem>" || std::path::Path::new(arg).is_file())
                && !arg.starts_with('-')
                && (i == 0 || many)
            {
                true => resolve(arg),
                false => Cow::Borrowed(arg),
            },
        )
        .collect();
    newest.map(Cow::Owned).into_iter().chain(named).collect()
}

/// The most recently edited `.cp` file in `dir`, the file a command that
/// takes one reads when none is named (spec/tools/cli.md).
pub(crate) fn newest_source(dir: &std::path::Path) -> Option<String> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|e| e == "cp"))
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.file_name())))
        .max()
        .map(|(_, name)| name.to_string_lossy().into_owned())
}

/// The first argument that looks like an option (`-x`, `--x`; a lone `-` is
/// a path) but is none of the command's, for the commands parsed here.
pub(crate) fn unknown_option<'a>(command: &str, args: &[&'a str]) -> Option<&'a str> {
    let options = COMMANDS.iter().find(|c| c.name == command)?.options?;
    args.iter()
        .copied()
        .filter(|arg| !matches!(*arg, "-h" | "--help"))
        .find(|arg| arg.len() > 1 && arg.starts_with('-') && !options.contains(arg))
}

/// The `a.cp` a bare `a` names: when there is no `a`, or `a` is not text
/// (the binary `g++ -o a` left beside it).
fn bare_source(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let cp = path.with_extension("cp");
    let shadowed = || {
        path.is_file()
            && std::fs::read(path).is_ok_and(|bytes| std::str::from_utf8(&bytes).is_err())
    };
    (path.extension().is_none() && cp.is_file() && (!path.exists() || shadowed())).then_some(cp)
}

#[cfg(test)]
mod tests {
    use super::{bare_source, long_forms, Json, COMMANDS};

    #[test]
    fn a_bare_name_is_its_source_even_beside_its_binary() {
        let dir = std::env::temp_dir().join(format!("cpl-bare-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let name = dir.join("cf2272A");
        assert_eq!(bare_source(&name), None, "no source, no binary");
        std::fs::write(dir.join("cf2272A.cp"), "print(1)\n").unwrap();
        assert_eq!(bare_source(&name), Some(dir.join("cf2272A.cp")));
        std::fs::write(&name, [0x7f, b'E', b'L', b'F', 0xff, 0xfe]).unwrap();
        assert_eq!(bare_source(&name), Some(dir.join("cf2272A.cp")), "a binary");
        std::fs::write(&name, "print(2)\n").unwrap();
        assert_eq!(bare_source(&name), None, "a text file is taken as written");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn short_forms_name_their_long_forms() {
        assert_eq!(
            long_forms(&["b", "a.cp", "-c"]),
            ["build", "a.cp", "--copy"]
        );
        assert_eq!(long_forms(&["submit", "-c"]), ["submit", "--copy"]);
        assert_eq!(long_forms(&["r", "a"]), ["run", "a"]);
        assert_eq!(long_forms(&["t", "a"]), ["test", "a"]);
        assert_eq!(
            long_forms(&["run", "-c"]),
            ["run", "-c"],
            "`-c` is `--copy` only where it copies"
        );
        for (short, long) in super::SHORT_COMMANDS {
            assert!(
                COMMANDS.iter().any(|c| c.name == *long)
                    && !COMMANDS.iter().any(|c| c.name == *short)
            );
        }
        for (command, _, long) in super::SHORT_OPTIONS {
            let options = COMMANDS
                .iter()
                .find(|c| c.name == *command)
                .unwrap()
                .options
                .unwrap();
            assert!(options.contains(long), "{command} {long}");
        }
    }

    #[test]
    fn an_unknown_command_names_the_one_meant() {
        assert_eq!(super::guess("listen").as_deref(), Some("new --listen"));
        assert_eq!(super::guess("compile").as_deref(), Some("build"));
        assert_eq!(super::guess("tset").as_deref(), Some("test"));
        assert_eq!(super::guess("strss").as_deref(), Some("stress"));
        assert_eq!(super::guess("frobnicate"), None);
        assert_eq!(super::guess("x"), None, "too short to guess from");
        for (_, command) in super::OTHER_SPELLINGS {
            let name = command.split(' ').next().unwrap();
            assert!(COMMANDS.iter().any(|c| c.name == name), "{command}");
        }
    }

    #[test]
    fn a_parsed_command_accepts_its_own_json_spelling() {
        for c in COMMANDS {
            let Some(options) = c.options else { continue };
            match c.json {
                Json::Flag => assert!(options.contains(&"--json"), "{}", c.name),
                Json::Format => assert!(options.contains(&"--format"), "{}", c.name),
                Json::None => {}
            }
        }
    }
}
