# cli-tools

Segment: C. Design, then build. Owns: `crates/cpl-cli`, `cpl-contest`, `cpl-doc`, `cpl-json`, `spec/tools/{cli,bundle,explain}.md`. Worklog: [worklog.md](worklog.md).

## Covers
The `cpl` CLI and every command no other lane owns: `run`, `test`, `watch`, `check`, `fix`, `bundle`, `submit`, `explain`, `profile`, `cover`, `bench` (harness; perf owns the numbers), `doc`, `self` (toolchain manager, pins, `settings.toml`), contest tools (`cpl-contest`), the JSON/machine output every tool and editor reads. Installers stay with release; stress/lint/fmt/lsp/trace/judges commands stay with their lanes.

## Improve
Audit every command against `spec/tools/cli.md` on v0.3 (flags, exit codes, help text, one vocabulary); `bundle` always yields one C++17 file under 32 KB; `explain` covers every implicit choice; `self` switches stable/edge/dated toolchains and rebuilds old solutions with their pin; `cpl --help` and `cli.json` generated from one source (fixes the site's missing `cli.json`); startup and front end < 500 ms.

## Success criteria
Every command in `cli.md` has a golden test of its help, output and exit code; 0 spec/CLI mismatches (generated check); every corpus port bundles under 32 KB; `explain` shows every implicit representation choice the conformance suite exercises; `cpl self` installs, switches and removes toolchains in an install test on Linux, macOS and Windows; p95 front end < 500 ms on the corpus.

## Ryan's role
Reviews the command set, names and the toolchain model (`self`, pins); after that the spec is the contract.
