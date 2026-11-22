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

# Format stdlib and check Rust formatting on changed files.
fmt:
    cargo build -p cpl-cli && target/debug/cpl fmt stdlib

# Check formatting without modifying files.
fmt-check:
    cargo build -p cpl-cli && target/debug/cpl fmt --check stdlib

# Run the max-N C++ benchmark suite (quiet machine recommended).
bench *args:
    cargo build --release -p cpl-cli
    python3 bench/corpus_p95_bench.py {{args}}

# Build and verify the offline docs + playground bundle.
docs:
    python3 site/bundle.py
    python3 site/check_docs.py

# Build the offline site bundle and run site unit tests.
site:
    python3 site/bundle.py
    python3 site/test_site.py

# Run spec conformance checks.
conformance *args:
    python3 scripts/conformance.py {{args}}

# Check corpus metadata integrity.
corpus-check:
    python3 scripts/corpus.py check

# Create an isolated git worktree with its own Cargo target directory.
worktree name:
    git worktree add "../cp-lang-{{name}}" -b "{{name}}"
    @echo "Worktree created at ../cp-lang-{{name}} (set CARGO_TARGET_DIR=../cp-lang-{{name}}/target)"
