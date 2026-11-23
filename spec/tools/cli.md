# The `cpl` command

Status: draft for 0.3.0. Sources: `plan/40-tooling.md` §8, decisions 34,
36. Every command takes a file (`d`, `d.cp`, a path) and defaults to the
most recently edited `.cp` file in the folder. A path may use `/` on every
platform and `\` on Windows too; `d.tests/` sits beside `d.cp`.

| Command | Does |
|---|---|
| `cpl new d` | creates `d.cp` from the template and `d.tests/`; `--json` prints `{"path", "created", "tests", "samples"}` as `cpl fetch --json` does |
| `cpl fetch <url> [d]` | downloads the samples of a Codeforces, AtCoder or CSES problem page into `d.tests/`, and creates `d.cp` with the title and limits if it is new; `--html` reads a page saved from the browser (Codeforces refuses scripts); a contest's page fetches each problem it lists into a folder named after the contest (`abc300/a.cp`, ...), `--json` printing `{"contest", "folder", "problems"}` |
| `cpl check d...` | every error in the files: names, types, borrows, input ranges; nothing runs |
| `cpl fix d...` | applies each error's suggested fix and reports what is left |
| `cpl run d [files]` | checked build in the interpreter, on files or stdin; with neither (stdin a terminal) it runs every sample in `d.tests/` as `cpl test d` does, and `cpl run d 2` runs the sample `2`; compares with `.out` files if present; `--release` runs the judge's C++ (`--checks` as for `build`; traps by default); `--trace` writes the HTML trace (`tools/trace-debug.md`); `--time-limit MS` stops the run after MS milliseconds; on a file with an `@interactor` it runs the program against it |
| `cpl test d` | every case in `d.tests/` (`1.in` with an optional `1.out`, numbered from 1) and `@test` fns; with an `@interactor`, `N.in` is the hidden test and the interactor's verdict decides, so `N.out` is not needed; `--case 2` runs one; `--add FILE [--expect OUT]` adds the next numbered case and runs it; output matches `.out` token by token, except that two tokens which both parse as numbers, either with a `.` or an exponent, match within \(10^{-6}\) absolute or relative error (`0.625` is `0.625000000`), and a `@check` overrides both; `submit` and `watch` compare the same way |
| `cpl stress d` | alternatives, generators, checkers, law sampling; shrinks and saves failures (`tools/stress.md`) |
| `cpl locate d` | the smallest failing input and the first part whose alt disagrees |
| `cpl judge d input [output answer]` | runs the file's `@interactor` as a judge program, or its `@check` on an output; exit 0 accepts |
| `cpl explain d [name]` | layouts, widths, niches, peak memory, specializations, checks, copies (`tools/explain.md`) |
| `cpl replay d --trace t.json --step n` | runs one recorded call again with the file's edited fn |
| `cpl dap` | the time-travel debug adapter (DAP) on stdio, the editors' debugger over the interpreter |
| `cpl profile d input` | per-line time and allocation on the release C++ |
| `cpl cover d` | which lines and branches the tests reach |
| `cpl bench d` | release runtime against the profile's limits and against a matched C++ file if given |
| `cpl fmt [files]` | canonical formatting (`tools/fmt-lint.md`) |
| `cpl lint [files]` | lints with autofixes |
| `cpl build d` | the C++ for the file's judge, written to `d.cpp` beside the source with its path and size on stdout (`-o out`; `-o -` writes the C++ to stdout); `--checks none\|traps\|all` (what the judge build still traps on; `none` by default, decisions.md 299), `--checked`, `--compiler <toolchain>` (builds with that toolchain through the launcher, as `cpl +<toolchain> build` does; R11.4.2), `--size`, `--judge`, `--copy` (`-c`); `cpl b -c` is `cpl build --copy`, and `cpl r`, `cpl t` are `run`, `test` |
| `cpl submit d` | `build`, then the cases on that build with the judge's flags; `--copy` (`-c`) puts the file on the clipboard |
| `cpl migrate [path]...` | rewrites spellings newer versions removed (each `E0503` with a fix) in every `.cp` file under the paths, shows the diff, and lists what needs a hand edit; `--check` changes nothing. Stdlib codemods join as their diagnostics land; recording pins waits on `cp.toml` pins |
| `cpl judges` | lists judge profiles |
| `cpl doc` | renders library docs from doc comments |
| `cpl lsp` | the language server |
| `cpl watch d` | rebuild and test on save |
| `cpl report d` | writes a paste-ready bug report (`--last` after an internal compiler error) |
| `cpl setup neovim` | installs the editor support built into `cpl` (`neovim`, `vim`) where the editor loads it at startup; `vscode` installs the release extension through `code --install-extension`; `jetbrains` unpacks the release plugin into each installed IDE |
| `cpl self <command>` | updates, switches or rolls back the installed toolchain; `cpl +<toolchain> <command>` runs one command with another |

Output is plain text for people and `--json` (the same as `--format json`, on every command that has
JSON output) for tools; exit codes are
0 success; 1 the files have errors or a check failed (compile errors, a
failing test case, `fmt --check` or `lint` findings); 2 usage error, as
POSIX tools and getopt use it; 3 the program itself failed (a trap, a failed
`assert`, a nonzero exit), the same code the emitted C++ exits with. (Rcli.1) An option a command
doesn't have reads the same on every command,
``cpl: `test` has no option `--loud`; `cpl test --help` lists them``, then that command's usage. Everything is deterministic given the seed.

`cpl check --json` prints `{"files": [{"path", "ok", "diagnostics"}]}`; each
diagnostic has `code`, `severity`, `message`, `start`, `end`, `labels`,
`notes`, `help` and `fixes`, positions as 1-based `line` and byte `col` plus
the byte `offset`. (Rcli.2) `cpl fix --json` prints the same `files`, each also with
the number of fixes `applied`, its diagnostics those left in the rewritten
file. `cpl judge --json` prints `{"accepted", "reason", "diagnostic"}`, the
last two `null` when the judge accepts. `cpl build --json` prints the C++
file's `path`, `bytes`, the `limit` and `judge` it was built for, and `sizes`:
each piece's `part`, `name` and `bytes`, adding up to `bytes`. `cpl submit
--json` prints `ready` and the `build` and `test` reports those two print,
`null` for a step that did not run. `cpl report --json` prints the report's
`path` and `bytes`, its `sections`, the `comparison` of the interpreter and
C++ runs and the prefilled `issue_url`, `null` when there is none. `cpl fmt
--json` prints `{"files": [{"path", "changed", "ok", "diagnostics",
"error"}]}` in path order: `changed` when formatting changed the file (or
would, under `--check`), `ok` when it is formatted, the lexer `diagnostics`
that stopped formatting and any other `error` (`null` when none).
`cpl setup <editor> --json` prints `{"editor", "ok", "message"}`, or
`"error"` in place of `"message"` when the install or `--check` failed;
`editor` is the canonical name (`code` reports `vscode`).
`cpl self list --json` prints `{"toolchains": [{"id", "marks"}]}`, the
marks among `stable`, `edge`, `default` and `rollback`; `cpl self channel
--json` prints `{"default", "toolchain"}`, `toolchain` `null` until it is
installed; every other `cpl self` command prints `{"ok", "message"}` or
`{"ok", "error"}`.
`cpl watch --json` prints each round as one line, `{"path", "passed",
"diagnostics", "test", "skipped", "error"}`: the errors (or, with none, the
lints) in `cpl check --json`'s diagnostic shape, `test` as `cpl test --json`
prints it or `null` when the cases did not run, why they were `skipped`, and
any `error` that stopped the round (`null` when none).
`cpl --help --json` prints `{"commands": [{"name", "usage", "summary",
"json", "help"}]}`: `json` is how the command asks for JSON output
(`"--json"` or `"--format json"`; both work) or `null` when it has none.
