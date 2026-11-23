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
}

pub(crate) const COMMANDS: &[Command] = &[
    Command {
        name: "check",
        args: "<file>...",
        summary: &["report every error in the files"],
    },
    Command {
        name: "fix",
        args: "<file>...",
        summary: &["apply the fix each error suggests, then report what is left"],
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
    },
    Command {
        name: "judge",
        args: "<file> <input> [<output> <answer>]",
        summary: &[
            "run the file's `@interactor` as a judge program on",
            "standard input and output, or with an output its",
            "`@checker`; exit status 0 accepts",
        ],
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
    },
    Command {
        name: "judges",
        args: "[file]",
        summary: &["list the judge profiles, marking the one a build uses"],
    },
    Command {
        name: "submit",
        args: "<file>",
        summary: &[
            "build for the file's judge, run its cases on that",
            "build, and say whether it is ready to send",
        ],
    },
    Command {
        name: "fmt",
        args: "[path]...",
        summary: &["format files in the one canonical style"],
    },
    Command {
        name: "lint",
        args: "[path]...",
        summary: &["warn about code that is legal but probably wrong or slow"],
    },
    Command {
        name: "explain",
        args: "<CODE|file|--json>",
        summary: &[
            "show a diagnostic or lint explanation, or a file's",
            "representations and view costs",
        ],
    },
    Command {
        name: "doc",
        args: "[path]...",
        summary: &["build HTML documentation from `///` comments"],
    },
    Command {
        name: "new",
        args: "<name>",
        summary: &["create a problem from its constraints or samples"],
    },
    Command {
        name: "test",
        args: "<problem>",
        summary: &["run every case in <problem>.tests/"],
    },
    Command {
        name: "stress",
        args: "<file>",
        summary: &["run against a brute on random inputs; shrink the failure"],
    },
    Command {
        name: "bench",
        args: "<problem>",
        summary: &["run on maximum-size worst-case inputs against the limits"],
    },
    Command {
        name: "profile",
        args: "<file> [input]",
        summary: &["run, then show each line's hits and share of the time"],
    },
    Command {
        name: "cover",
        args: "<file> [input]...",
        summary: &["list the lines and branches no input reached"],
    },
    Command {
        name: "watch",
        args: "<problem>",
        summary: &["check, lint and test on every save"],
    },
    Command {
        name: "report",
        args: "<file>",
        summary: &["write a paste-ready bug report (`--last` after an ICE)"],
    },
    Command {
        name: "lsp",
        args: "",
        summary: &["serve the language server protocol on stdio"],
    },
    Command {
        name: "dap",
        args: "",
        summary: &["serve the time-travel debug adapter (DAP) on stdio"],
    },
    Command {
        name: "replay",
        args: "<file> --trace <t.json> --step <n>",
        summary: &["run one recorded call again with the file's edited fn"],
    },
    Command {
        name: "self",
        args: "<command>",
        summary: &[
            "update, switch or roll back the installed toolchain",
            "(`cpl self --help`); `cpl +<toolchain> <command>` runs",
            "one command with another, e.g. `cpl +edge run a.cp`",
        ],
    },
];

const HEADER: &str = "\
usage: cpl <command> [args]

commands:
";

const FOOTER: &str = "
`cpl <command> --help` describes a command's options.

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
        "run" => RUN_USAGE,
        "build" => BUILD_USAGE,
        "judge" => JUDGE_USAGE,
        "submit" => SUBMIT_USAGE,
        "judges" => JUDGES_USAGE,
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

Reports every error in the files: names, types, borrows, input ranges.
Nothing is run and nothing is written. The exit status is 1 when a file
has an error.

  --json    print one JSON object instead: `files`, each with its `path`,
            `ok`, and `diagnostics` (code, severity, message, start, end,
            labels, notes, help, fixes; positions are 1-based line and byte
            column, with the byte offset)
";

const FIX_USAGE: &str = "\
usage: cpl fix <file.cp>...

Applies the fix each error carries (the quick fixes an editor offers),
then, once no error is left, the fixes `cpl lint --fix` makes. Rewrites the
file and reports the errors left. The exit status is 1 when an error is left.
";

const BUILD_USAGE: &str = "\
usage: cpl build <file.cp> [options]

Checks the file, then writes one C++17 file to submit, next to the source.

options:
  -o <out>          write to <out> instead; `-o -` writes to standard output
  --judge <name>    for that judge; else the file's `// judge:` line, else
                    `judge` under `[judge]` in the nearest cp.toml, else
                    `generic`. `cpl judges` lists them
  --checked         also compile it with sanitizers into a program to run here
  --size            say where the bytes go: header, runtime, library fns,
                    your code, by part and by piece (said anyway when the
                    file is over the judge's limit)
  --copy            also put the C++ on the clipboard (pbcopy, wl-copy, xclip,
                    xsel or clip, else the command in CPL_CLIPBOARD)

Over 32 KB it warns; over 64 KB, or the judge's own smaller limit, it fails
and says where the bytes go. The same file and judge give the same bytes.
";

const JUDGES_USAGE: &str = "\
usage: cpl judges [file.cp]

Lists the judge profiles: compiler, flags, stack and source limit. With a
file, marks the one its build uses (the `// judge:` line, else the nearest
cp.toml, else `generic`).
";

const EXPLAIN_USAGE: &str = "\
usage: cpl explain <CODE | file.cp | --json>

With a code such as E0302, explains that diagnostic or lint: what triggers
it and how to fix it. With a file, shows how each value is represented and
what its views cost. `--json` prints every explanation as JSON.
";

const SUBMIT_USAGE: &str = "\
usage: cpl submit <file.cp>

Gets the file ready to send to its judge (the `// judge:` line, else the
nearest cp.toml, else `generic`): writes the C++ as `cpl build` does, within
the judge's source limit, then runs every case in <file>.tests/ on that C++
built with the judge's flags on its stack, as `cpl test` does. Exits 0 when
the file is ready: it built and no case failed.

options:
  --copy            when ready, also put the C++ on the clipboard, as
                    `cpl build --copy` does

Mid-contest: put `// judge: codeforces` (or another of `cpl judges`) at the
top of the file once, save the samples as <file>.tests/1.in and 1.out, then
`cpl submit a.cp --copy` and paste, or upload a.cpp.
";

const JUDGE_USAGE: &str = "\
usage: cpl judge <file.cp> <input>
       cpl judge <file.cp> <input> <output> [answer]

With an input alone, runs the file's `@interactor` on it as a judge
program: it talks to a solution on standard input and output. With an
output, runs the file's `@checker` on that output. The exit status is 0
when the judge accepts.
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
usage: cpl run <file.cp> [input] [options]

Checks the file, then runs it in the debug interpreter on the input file
or on standard input. What `debug(...)` shows goes to standard error.

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
  --depth N         with --trace: record only up to call depth N (0 = top level)
  --format json     without --trace: the value of each top-level variable,
                    the output and the time, for an editor.
                    with --trace: the trace as JSON on standard output
  --format html     with --trace: the trace viewer with the trace in it,
                    one self-contained page on standard output
  --live            with --trace: serve the viewer on localhost and record
                    again each time the file is saved, on the same input;
                    with no input named, the case `cpl stress` saved last.
                    The page keeps its place and shows the first change
  --brute           run the file's `@brute(main)` fn in place of the solution
  --release         run what a judge runs: the C++ of `cpl build`, built with
                    the judge's flags (those g++ here accepts) on the judge's
                    stack, and kept until the file changes.
                    Fast, and without the interpreter's checks
";

/// The first argument that looks like an option (`-x`, `--x`; a lone `-` is
/// a path) but is none of the command's, for the commands parsed here.
pub(crate) fn unknown_option<'a>(command: &str, args: &[&'a str]) -> Option<&'a str> {
    let options: &[&str] = match command {
        "check" => &["--json"],
        "fix" | "judge" | "judges" => &[],
        "build" => &["-o", "--judge", "--checked", "--size", "--copy"],
        "submit" => &["--copy"],
        "explain" => &["--json"],
        "profile" | "cover" => &["--native", "--format"],
        "lsp" | "dap" => &["--stdio"],
        _ => return None,
    };
    args.iter()
        .copied()
        .filter(|arg| !matches!(*arg, "-h" | "--help"))
        .find(|arg| arg.len() > 1 && arg.starts_with('-') && !options.contains(arg))
}
