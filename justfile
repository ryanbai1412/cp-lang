# CP Lang task runner. Run `just` to list recipes.
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# The fast gate: what CI runs on every main tip.
check:
    scripts/check.sh --fast

# The commit tier (the pre-commit hook): only what the staged paths reach.
check-commit:
    scripts/check.sh --commit

# Full pre-push / release gate (adds prover fuzz, budgets, wasm smoke, VS Code tests).
check-full:
    scripts/check.sh --full

# The wall-clock budgets (CI: ci-slow `timing`; they report, never block).
check-timing:
    scripts/check.sh --timing

# The freeze's behaviour half, when a path since `base` reaches the old pipeline
# (scripts/affected.py's old_same): its output on every case is as at `base`.
old-same base:
    if uv run --quiet python scripts/affected.py --base "{{base}}" | grep -qx 'old_same=1'; then uv run --quiet python scripts/old-pipeline-same.py --base "{{base}}"; else echo "old-same: no old-pipeline input changed since {{base}}"; fi

# Run one normative spec gate, e.g. `just spec-gate mutants --n 12`.
spec-gate *args:
    uv run --quiet python scripts/spec-gate.py {{args}}

# Which tests cover which rule (CI: ci-slow `coverage`).
coverage-matrix *args:
    uv run --quiet python scripts/coverage-matrix.py {{args}}

# The mutation score of one file against its tests (CI: ci-slow `dap-mutants`).
mutation-score *args:
    uv run --quiet python scripts/mutation-score.py {{args}}

# Every std conformance case and std doc example as C++ under ASan, UBSan and
# _GLIBCXX_DEBUG; fails on any report (CI: ci-slow `std-sanitize`).
std-sanitize *args:
    cargo run --release -q -p cpl-conform -- sanitize {{args}}

# Every pair of std views, keys and ops, lazy against materialized, on both
# engines (std-api/R14; not in CI until it is green).
std-compose *args:
    uv run --quiet python scripts/std-compose.py {{args}}

# LSP request latency, in release (CI: ci-slow `lsp-latency`).
lsp-latency:
    cargo test --release -p cpl-lsp --test latency -- --nocapture

# `cpl` cold-start time, in release (CI: ci-slow `lsp-latency`).
cold-start:
    cargo test --release -p cpl-cli --test cold_start -- --nocapture

# The CLI's golden and contest tests (CI: ci.yml `cli`).
test-cli:
    cargo test -p cpl-cli --lib --test golden --test cli --test clipboard
    cargo test -p cpl-contest

# `cpl b -c` through this OS's real clipboard and back (CI: ci.yml `cli`).
clipboard-roundtrip cpl="target/debug/cpl":
    bash scripts/clipboard-roundtrip.sh "{{cpl}}"

# Each editor's smoke test against the built `cpl` (CI: ci.yml `neovim`, `editors`).
neovim-smoke cpl="target/debug/cpl":
    nvim --headless --clean -l editors/neovim/test/smoke.lua "$(realpath {{cpl}})" "$PWD"

vim-smoke cpl="target/debug/cpl":
    bash editors/neovim/test/smoke_vim.sh "$(realpath {{cpl}})"

# Emacs on this OS, for emacs-smoke and the fresh-home setup test (CI: ci.yml `tests`, `cli`, `editors`).
install-emacs:
    bash scripts/install-emacs.sh

emacs-smoke cpl="target/debug/cpl":
    CPL="$(realpath {{cpl}})" emacs --batch -Q -l editors/emacs/smoke.el
    uv run --quiet python editors/lsp-smoke.py emacs "$(realpath {{cpl}})" lsp

helix-smoke cpl="target/debug/cpl":
    editors/helix/smoke.sh "$(realpath {{cpl}})"

zed-smoke cpl="target/debug/cpl":
    editors/zed/smoke.sh "$(realpath {{cpl}})"

# Parse the tree-sitter grammar's smoke corpus.
tree-sitter-smoke:
    editors/tree-sitter-cp/smoke.sh

# Every generated grammar against the lexer over the conformance cases; `engines` (comma-separated,
# e.g. `vim,emacs`) must run there, not be skipped (CI: ci.yml `neovim`, `editors`).
highlight-agree engines="":
    CPL_HIGHLIGHT={{engines}} cargo test -q -p cpl-cli --test highlight_agrees

# The VS Code extension's unit, grammar and host tests (CI: ci.yml `vscode`).
vscode-test: && vscode-integration
    cd editors/vscode && npm ci --no-audit --no-fund && npm test

# The VS Code extension's tests in a real host, under xvfb when there is one.
vscode-integration:
    cd editors/vscode && npm ci --no-audit --no-fund
    cd editors/vscode && if command -v xvfb-run >/dev/null; then xvfb-run -a npm run test:integration; else npm run test:integration; fi

# The stress and time-travel debugger tests (CI: debugger-loop.yml `loop`).
debugger-loop:
    cargo nextest run --no-fail-fast -p cpl-stress -p cpl-cli --test debugger --test runtime_errors --test stress_bench

# Neovim's stress loop and nvim-dap time travel against debug cpl (CI: debugger-loop.yml `loop`).
neovim-stress-loop tmp="target/tmp":
    #!/usr/bin/env bash
    set -euo pipefail
    root=$PWD cpl=$PWD/target/debug/cpl dap="{{tmp}}/nvim-dap"
    if command -v cygpath >/dev/null; then root=$(cygpath -m "$PWD"); cpl="$root/target/debug/cpl.exe"; fi
    nvim --headless --clean -l editors/neovim/test/stress_loop.lua "$cpl" "$root"
    [ -d "$dap" ] || git clone -q --depth 1 --branch 0.9.0 https://github.com/mfussenegger/nvim-dap "$dap"
    if command -v cygpath >/dev/null; then dap=$(cygpath -m "$dap"); fi
    nvim --headless --clean -l editors/neovim/test/dap_loop.lua "$cpl" "$root" "$dap"

vim-stress-loop cpl="target/debug/cpl":
    editors/neovim/test/stress_loop_vim.sh "$(realpath {{cpl}})"

# Every workflow and action parses, with valid expressions, action inputs and shellcheck-clean
# run scripts (CI: the
# checks gate). A workflow GitHub can't parse runs no jobs, so nothing else reports it.
workflow-lint:
    command -v shellcheck >/dev/null || { echo "workflow-lint: needs shellcheck on PATH"; exit 1; }
    actionlint

# The JetBrains plugin (CI: jetbrains.yml, release.yml). The fetch is the one retried
# step, network only; the build and tests then run once, offline.
jetbrains-fetch *flags:
    cd editors/jetbrains && . ../../scripts/retry.sh && retry ./gradlew --no-daemon --init-script ../../scripts/gradle-mirror.init.gradle prefetch {{flags}}

# Run Gradle tasks in the JetBrains plugin, offline against the prefetched mirror.
jetbrains-gradle *tasks:
    cd editors/jetbrains && ./gradlew --no-daemon --init-script ../../scripts/gradle-mirror.init.gradle --offline {{tasks}}

# Verify the built plugin against IntelliJ IDEA Community and Ultimate (downloads both).
jetbrains-verify:
    cd editors/jetbrains && ./gradlew --no-daemon --init-script ../../scripts/gradle-mirror.init.gradle verifyPlugin

# The LSP handshake the JetBrains plugin starts, against debug cpl.
jetbrains-handshake cpl="target/debug/cpl":
    uv run --quiet python editors/lsp-smoke.py jetbrains {{cpl}} lsp

# The acceptance regression suite, e.g. `just acceptance target/ci/cpl --shard 1/3`.
acceptance *args:
    uv run --quiet python research/acceptance/regress.py {{args}}

# Follow getting-started from a fresh install on this OS (CI: acceptance.yml `install-cli`).
acceptance-steps os:
    research/acceptance/ci-steps.sh "{{os}}"

# Run the Rust workspace test suite via cargo-nextest (or cargo test).
[positional-arguments]
test *args:
    cargo nextest run --workspace "$@"

# Run zero-cost abstraction + corpus runtime (including perf.cp) benchmarks (not in fast gate).
bench *args:
    cargo build --release -p cpl-cli
    target/release/cpl bench --zero-cost bench/zero-cost {{args}}
    target/release/cpl bench --corpus {{args}}

# Run only the zero-cost abstraction benchmark suite.
bench-zero-cost *args:
    cargo build --release -p cpl-cli
    target/release/cpl bench --zero-cost bench/zero-cost {{args}}

# Run only the corpus runtime benchmark suite (writes research/perf/corpus-p95-results.json).
bench-corpus *args:
    cargo build --release -p cpl-cli
    target/release/cpl bench --corpus {{args}}

# Build the Next.js web app (web/).
web-build:
    pnpm --dir web build

# Typecheck and unit-test the web app (CI: ci.yml `site`).
web-typecheck:
    pnpm --dir web typecheck

# Run the web workspace's tests.
web-test:
    pnpm --dir web test

# cpl, the browser build and its named copy for wasm-size, in parallel (CI: ci.yml `site`).
site-binaries out="target/site-binaries":
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "{{out}}"
    scripts/build-wasm.sh >"{{out}}/wasm.log" 2>&1 & wasm=$!
    scripts/build-wasm.sh --names "{{out}}/wasm-names" >"{{out}}/wasm-names.log" 2>&1 & names=$!
    cargo build --profile ci --bin cpl
    wait "$wasm" || { cat "{{out}}/wasm.log"; exit 1; }
    wait "$names" || { cat "{{out}}/wasm-names.log"; exit 1; }
    cat "{{out}}/wasm.log"

# The shrink-only cpl.wasm ratchet per main tip (CI: ci.yml `site`): record|check|lower.
wasm-ratchet cmd wasm sizes:
    scripts/wasm-ratchet.sh {{cmd}} "{{wasm}}" "{{sizes}}"

# A named cpl.wasm of HEAD in the background, for land.sh's wasm budget check (scripts/wasm-warm.sh).
wasm-warm:
    scripts/wasm-warm.sh

# Run the browser smoke suite against the built site (web/out), on a port of its own.
web-smoke:
    pnpm --dir web smoke

# Build the site, hold cpl.wasm to its size cap and run the browser smoke suite; skipped when unchanged.
site *args:
    scripts/site-check.sh {{args}}

# Run the unified conformance/corpus/docs/diff harness (crates/cpl-conform).
conform *args:
    cargo run -p cpl-conform -- {{args}}

# Alias for `just conform`.
conformance *args:
    cargo run -p cpl-conform -- {{args}}

# Write cpl-next's case lists from one CI run's 16 lists@ shards (the run id names the regen).
next-lists-from-run run:
    uv run python scripts/next-lists-from-run.py {{run}}

# Install local git pre-commit and commit-msg hooks.
hooks:
    scripts/install-hooks.sh

# Reserve the next decision number on main and print it (needs CPL_LANE).
next-decision title:
    uv run python scripts/spec-gate.py decisions --reserve --title "{{title}}"

# Check that repo paths cited in live docs exist.
check-links:
    uv run python scripts/check-doc-links.py

# Check corpus metadata integrity.
corpus-check:
    uv run python scripts/corpus.py check

# Write target/corpus-tests/<id>/NN.{in,ans} from each tests.toml and accepted C++.
corpus-tests-make *args:
    uv run python scripts/corpus.py make {{args}}

# Check each v03 --max input reaches its port's Bounds against research/corpus-max-short.json (--write records it).
corpus-max-short *args:
    uv run python scripts/corpus.py max-short {{args}}

# Check each dream.cp's C++ is under 32 KB against research/dream-over-32kb.json (--write records it).
corpus-dream-sizes *args:
    uv run python scripts/corpus.py dream-sizes {{args}}

# Drop the g++ cache's binaries unused since `since` (epoch seconds); CI runs it before saving the cache.
cxx-trim since *dirs:
    uv run python scripts/ci.py cxx-trim --since {{since}} {{dirs}}

# Run every corpus mode on packages: header, check, package, tests, stress, max, size, lint, dream.
corpus-verify *args:
    uv run python scripts/corpus.py verify {{args}}

# Run the normative spec checks (stub, vocab, dream gaps).
spec-check:
    scripts/spec-check.sh

# Refresh or check the conformance coverage table.
conformance-coverage *args:
    uv run python scripts/spec-gate.py coverage {{args}}

# Write plan/STATUS.md from every domain's metrics.toml and the open questions.
status:
    uv run python scripts/status.py

# Regenerate every committed generated file (`just gen --check`: fail if stale).
gen *args:
    uv run python scripts/gen.py {{args}}

# Check the tech-debt ratchet (`--show` lists every counter).
debt-ratchet *args:
    uv run python scripts/debt-ratchet.py --check {{args}}

# Fail on any counter that is not at 0 (ci-slow.yml's debt-zero job).
debt-zero:
    uv run python scripts/debt-ratchet.py --zero

# Create an isolated git worktree with its own Cargo target directory.
worktree name:
    git worktree add "../cp-lang-{{name}}" -b "{{name}}"
    @echo "Worktree created at ../cp-lang-{{name}} (set CARGO_TARGET_DIR=../cp-lang-{{name}}/target)"

# Land HEAD on main: rebase, the commit tier on the exact tip, push (ci/R27).
land:
    scripts/land.sh

# Which tests a change to these paths selects in the commit tier.
affected *paths:
    uv run python scripts/affected.py {{paths}}

# Bisect failing tests over a commit range and verify a revert.
culprit *args:
    uv run python scripts/culprit.py {{args}}

# Landing p50/p95 and commits per hour per domain.
land-stats *args:
    uv run python scripts/land_stats.py {{args}}

# Each lane's share of commits spent on devex, and velocity per 15 min.
devex-stats *args:
    uv run python scripts/devex_stats.py {{args}}

# The newest main commit no worse than the last edge on the shipping checks.
shippable:
    scripts/release/shippable.sh

# Run the scripts' own tests.
script-tests:
    uv run python -m unittest scripts/test_*.py
