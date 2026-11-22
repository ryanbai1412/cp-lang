# CP Lang task runner. Run `just` to list recipes.
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Fast local commit gate (wraps scripts/check.sh --fast).
check:
    scripts/check.sh --fast

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

# Run only the corpus runtime benchmark suite (writes bench/corpus-p95-results.json).
bench-corpus *args:
    cargo build --release -p cpl-cli
    target/release/cpl bench --corpus {{args}}

# Build the Next.js web app (web/).
web-build:
    pnpm --dir web build

# Run the Next.js web app smoke suite (expects web/out served on :3000).
web-smoke:
    pnpm --dir web smoke

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
    python3 scripts/conformance-coverage.py {{args}}

# Check the tech-debt ratchet (.rs file sizes, stdlib todo, #[ignore], pending tests).
debt-ratchet *args:
    python3 scripts/debt-ratchet.py --check {{args}}

# Create an isolated git worktree with its own Cargo target directory.
worktree name:
    git worktree add "../cp-lang-{{name}}" -b "{{name}}"
    @echo "Worktree created at ../cp-lang-{{name}} (set CARGO_TARGET_DIR=../cp-lang-{{name}}/target)"
