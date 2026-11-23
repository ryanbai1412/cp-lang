# The v0.3 CLI and editor experience

2026-11-22, tools lane. For Ryan's one review (segment C). Folded into `spec/tools/cli.md` once decided.

## The contest loop, as typed
```
cpl new a              # a.cp from the template, a.tests/
cpl fetch <url>        # samples into a.tests/                  (to build)
cpl run a              # check, run every sample in the interpreter, diff
cpl test a             # samples, @sample, @test fns, @law properties
cpl stress a           # vs @brute; shrinks; saves the case       (debugger lane)
cpl run a --trace      # HTML trace of the run                    (debugger lane)
cpl explain a          # what each value costs and why
cpl submit a --copy    # build for the file's judge, run the cases on that C++, copy
```
One verb per job, the file as the only required argument, the judge from the file (`// judge:`) or `cp.toml`. Anything a command can print, `--json` prints as one document with the same fields.

## Rules every command follows
- **Help is the contract.** `cpl --help` lists every command; `cpl <cmd> --help` lists every flag; both come from one table in `cpl-cli`, which also emits `cpl --help --json` for the site.
- **Exit codes**: 0 ok, 1 a run or test failed, 2 compile error, 3 usage error.
- **Usage errors say the right invocation**: `cpl: build takes one file; did you mean \`cpl build a.cp\`?`
- **Same words everywhere**: help, errors, hover and explain use `spec/GLOSSARY.md` terms.
- **Fast**: `check` under 100 ms p95 on the corpus (today 78.5 ms), editor requests under 100 ms at 5,000 lines.

## Command set (decisions.md 244)
| Keep / add | Fold | Drop |
|---|---|---|
| new, fetch*, run, test, stress, locate*, explain, profile, cover, bench, fmt, lint, check, fix, build, submit, judge, judges, doc, lsp, dap, watch, migrate*, replay, report, self | `trace` → `run --trace`; `debug` → `dap` + editor; `interact` → `run` on a file with `@interactor` | `selftest` (it is `scripts/check.sh`) |

\* to build for v0.3.

## Editor
- `cpl lsp` is the one engine; editors are thin clients. Tier 1 (VS Code, Neovim): installed from the repo in one documented step, smoke-tested in the gate. Tier 2 (JetBrains, Zed, Helix, Emacs, Vim): smoke-tested when installed, fixed on report.
- Hover shows the same representation records as `cpl explain` (type, bytes, niche, checks left); inlay hints show proven ranges and sizes; code actions are the diagnostics' fixes (the same as `cpl fix`).
- Run/Trace/Stress/Replay code lenses call the CLI with `--json` and render its result.

## Explain (decisions.md 265)
v0.3: types and niches, arrays and peak memory, specializations chosen, checks left per build. After v0.3: contexts, generic instances, unsafe-tier sites. Missing sections print `not shown yet` rather than a guess.
