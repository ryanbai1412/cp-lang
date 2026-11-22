//! What `cpl --help` and each command's `--help` print, and which usage a
//! command's usage error shows.

pub(crate) const USAGE: &str = "\
usage: cpl <command> [args]

commands:
  check <file>...       report every error in the files
  fix <file>...         apply the fix each error suggests, then report what is left
  run <file> [input]    check, then run in the debug interpreter, reading
                        the input file or standard input; a file with an
                        `@interactor` runs against it, on the hidden test
                        (`cpl run --help` has the options for traces);
                        `--release` runs the C++ that `cpl build` writes
  judge <file> <input> [<output> <answer>]
                        run the file's `@interactor` as a judge program on
                        standard input and output, or with an output its
                        `@checker`; exit status 0 accepts
  build <file> [-o out] check, then write one C++17 file to submit: next to
                        the source by default, to standard output with `-o -`;
                        `--checked` also builds it with sanitizers to run here;
                        `--judge <name>` writes it for that judge
  judges [file]         list the judge profiles, marking the one a build uses
  submit <file>         build for the file's judge, run its cases on that
                        build, and say whether it is ready to send
  fmt [path]...         format files in the one canonical style
  lint [path]...        warn about code that is legal but probably wrong or slow
  explain <CODE|file|--json> show a diagnostic/lint explanation or a file's representations & view costs
  doc [path]...         build HTML documentation from `///` comments
  new <name>            create a problem from its constraints or samples
  test <problem>        run every case in <problem>.tests/
  stress <file>         run against a brute on random inputs; shrink the failure
  bench <problem>       run on maximum-size worst-case inputs against the limits
  profile <file> [input]
                        run, then show each line's hits and share of the time
  cover <file> [input]...
                        list the lines and branches no input reached
  watch <problem>       check, lint and test on every save
  report <file>         write a paste-ready bug report (`--last` after an ICE)
  lsp                   serve the language server protocol on stdio
  dap                   serve the time-travel debug adapter (DAP) on stdio
  replay <file> --trace <t.json> --step <n>
                        run one recorded call again with the file's edited fn
  self <command>        update, switch or roll back the installed toolchain
                        (`cpl self --help`); `cpl +<toolchain> <command>` runs
                        one command with another, e.g. `cpl +edge run a.cp`

`cpl <command> --help` describes a command's options.

options:
  -V, --version    print the version
  -h, --help       print this help
";

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
        _ => return None,
    })
}

const CHECK_USAGE: &str = "\
usage: cpl check <file.cp>...

Reports every error in the files: names, types, borrows, input ranges.
Nothing is run and nothing is written. The exit status is 1 when a file
has an error.
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
