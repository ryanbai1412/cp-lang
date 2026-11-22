# The `cpl` command

Status: draft for 0.3.0. Sources: `plan/40-tooling.md` §8, decisions 34,
36. Every command takes a file (`d`, `d.cp`, a path) and defaults to the
most recently edited `.cp` file in the folder.

| Command | Does |
|---|---|
| `cpl new d` | creates `d.cp` from the template and `d.tests/` |
| `cpl fetch <url>` | downloads samples into `d.tests/` |
| `cpl run d [files]` | checked build in the interpreter, on files or stdin; compares with `.out` files if present; `--release` runs the judge's C++ |
| `cpl test d` | every case in `d.tests/`, inline `@sample`s and `@test` fns |
| `cpl stress d` | alternatives, generators, checkers, law sampling; shrinks and saves failures (`tools/stress.md`) |
| `cpl locate d` | the smallest failing input and the first part whose alt disagrees |
| `cpl interact d` | runs against an `@interactor` or an external interactor binary |
| `cpl explain d [name]` | layouts, widths, niches, peak memory, specializations, checks, copies (`tools/explain.md`) |
| `cpl trace d input` | HTML trace of a run (`tools/trace-debug.md`) |
| `cpl debug d input` | the debugger over the interpreter |
| `cpl profile d input` | per-line time and allocation on the release C++ |
| `cpl cover d` | which lines and branches the tests reach |
| `cpl bench d` | release runtime against the profile's limits and against a matched C++ file if given |
| `cpl fmt [files]` | canonical formatting (`tools/fmt-lint.md`) |
| `cpl lint [files]` | lints with autofixes |
| `cpl build d` | the C++ for the file's judge; `--checked`, `--size`, `--judge` |
| `cpl submit d` | `build` plus size report, copies the file to the clipboard |
| `cpl migrate` | runs codemods to a new stdlib version and updates pins |
| `cpl judges` | lists judge profiles |
| `cpl doc` | renders library docs from doc comments |
| `cpl lsp` | the language server |
| `cpl watch d` | rebuild and test on save |
| `cpl selftest` | interpreter and C++ agreement on the corpus |

Output is plain text for people and `--json` for tools; exit codes are
0 success, 1 failure of the program or a test, 2 compile error, 3 usage
error. Everything is deterministic given the seed.
