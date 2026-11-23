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

# Run the Rust workspace test suite via cargo-nextest (or cargo test).
test *args:
    cargo nextest run --workspace {{args}}

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

# Install local git pre-commit and commit-msg hooks.
hooks:
    scripts/install-hooks.sh

# Print the next unused decision number in decisions.md.
next-decision:
    python3 scripts/next-decision.py

# Check that repo paths cited in live docs exist.
check-links:
    python3 scripts/check-doc-links.py

# Check corpus metadata integrity.
corpus-check:
    python3 scripts/corpus.py check

# Regenerate corpus/problems/<id>/tests/NN.{in,ans} from accepted C++ originals.
corpus-tests-make *args:
    python3 scripts/corpus-tests.py make {{args}}

# Run the normative spec checks (stub, vocab, dream-gaps).
spec-check:
    scripts/spec-check.sh

# Refresh or check the conformance coverage table.
conformance-coverage *args:
    uv run python scripts/conformance-coverage.py {{args}}

# Write plan/STATUS.md from every domain's metrics.toml and the open questions.
status:
    uv run python scripts/status.py

# Regenerate every committed generated file (`just gen --check`: fail if stale).
gen *args:
    uv run python scripts/gen.py {{args}}

# Check the tech-debt ratchet (`--show` lists every counter).
debt-ratchet *args:
    uv run python scripts/debt-ratchet.py --check {{args}}

# Create an isolated git worktree with its own Cargo target directory.
worktree name:
    git worktree add "../cp-lang-{{name}}" -b "{{name}}"
    @echo "Worktree created at ../cp-lang-{{name}} (set CARGO_TARGET_DIR=../cp-lang-{{name}}/target)"
