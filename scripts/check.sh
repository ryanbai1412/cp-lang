#!/usr/bin/env bash
# The checks to run before a commit or a push. There is no hosted CI on
# this repository: this script, run on your machine, is the gate.
#
#   scripts/check.sh           same as --fast
#   scripts/check.sh --commit  what the pre-commit hook runs: the checks the
#                              staged paths can reach (scripts/affected.py)
#   scripts/check.sh --land    --fast, but only the tests the paths changed
#                              since $CHECK_BASE (default origin/main) reach
#   scripts/check.sh --fast    what CI runs on every tip
#   scripts/check.sh --full    --fast, then the slow suites (15 minutes or more)
#
# --fast  1. nothing from dist/ and no secret is staged or in an unpushed commit
#         2. cargo clippy --workspace --all-targets -- -D warnings
#            and cargo check of cpl-wasm for wasm32 (the playground build)
#         3. the workspace tests (cargo nextest when installed, else cargo
#            test), without the tests listed under 8 and 9
#            (--fast compiles C++ through scripts/cxx-cache.py, which reuses
#            a binary built from the same source, flags and g++;
#            CHECK_NO_CXX_CACHE=1 turns it off)
#         4. the std/prelude.cp name count (decision 281: no budget)
#            and every port in corpus/problems passes cpl check,
#            cpl fmt --check and cpl lint with 0 warnings (under a second)
#            and cpl build writes at most 65,536 bytes of C++ for each
#         6. the count of open known_traps (crates/cpl-cli/tests/known_traps.cp)
#         7. the site's snippet gate (node web/scripts/gate.mjs): every
#            program on the site is at least at its declared status,
#            and the trace viewer's node tests (tools/trace/*.test.cjs)
# --full  (the steps are scripts/ci.py STEPS["full"])
#         8. the prover's products fuzzer, and 100 random programs
#            (scripts/gen-programs.py) against an independent model
#         9. the budget tests and the front-end latency test, each alone
#        10. the WebAssembly build and crates/cpl-wasm/tests/smoke.mjs
#            and scripts/site-check.sh: the site build (with the
#            gzipped cap on cpl.wasm in web/scripts/wasm-size.mjs) and its browser smoke
#        11. the VS Code extension tests (editors/vscode: npm test, and its
#            extension-host suite under a display or xvfb-run), then
#            editors/smoke.sh: Neovim, Emacs and Helix when installed
#
# --commit  1, rustfmt, clippy of the touched crates (and, when cpl-wasm
#           builds one of them, the wasm32 check and a names build whose
#           data segment must stay within its budget), and the nextest tests
#           of every crate that can reach them except the end-to-end suites
#           (profile `commit` in .config/nextest.toml), plus the unit tests
#           of touched scripts and the snippet gate for web/ and docs/. A
#           path scripts/affected.py does not know runs --fast instead.
#
# Needs cargo with clippy, python3 and node; --full also needs npm, pnpm, a
# Playwright Chromium, g++ and the wasm target (rustup target add wasm32-unknown-unknown).
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
exec uv run --quiet python scripts/ci.py check "$@"
