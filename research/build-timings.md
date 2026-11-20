# Rust build and test timings (0.2.2)

Measured 2026-11-20 on the Devin Linux VM: 8 cores, 31 GB RAM, rustc 1.100.0,
g++ 11.4, cargo-nextest 0.9.148. The workspace has no crates.io dependencies,
so every unit in these numbers is one of our own crates.

## Toolchain survey

- Linker: rustc already links x86_64 Linux with its bundled `rust-lld`
  (`readelf -p .comment target/debug/cpl` prints `Linker: LLD 23.1.5`), so a
  `-fuse-ld=lld` override would change nothing. `mold`, `sccache` and a system
  `clang` are not installed. With no external dependencies `sccache` has
  nothing to share across clones anyway.
- `cargo-nextest` is installed and runs the whole suite unchanged.

## What changed

`.cargo/config.toml` now sets the dev profile to `debug = 0` with
`split-debuginfo = "unpacked"` (and no debuginfo for build scripts), and adds
`cargo nt` as an alias for `cargo nextest run --workspace`. Panics and test
failures still print `file:line`; only symbolized backtraces lose line numbers.
Anyone who needs a debugger can set `debug = "line-tables-only"` locally.

Rejected after measuring: `opt-level = 1` in dev cut `cargo test --workspace`
from 94.4 s to 75.8 s but made a one-line `cpl-ir` rebuild 4.2 s instead of
0.9 s and a clean build 12.2 s instead of 5.2 s. Putting `opt-level = 1` only
in `[profile.test]` had the same cost on test builds (clean test build 21.0 s,
`cpl-ir` incremental test build 4.7 s) for a 78.9 s suite.

## Before and after

"Before" is the stock Cargo dev profile (`debug = true`, what a fresh clone
gets on a laptop); "after" is this repository's `.cargo/config.toml`. The two
were run interleaved in separate target dirs to cancel machine noise: clean
builds 3 times each, one-line edits (a new `pub fn` appended to
`crates/cpl-check/src/lib.rs` or `crates/cpl-ir/src/lib.rs`) 5 times each,
then the full suite once each. Means in seconds:

```
                                          before   after   change
cargo build --workspace (clean)             5.78    5.50     -5%
cargo test --workspace --no-run (clean)     5.56    3.98    -28%
1-line edit in cpl-check, cargo build       1.17    1.09     -7%
1-line edit in cpl-ir,    cargo build       1.23    1.08    -12%
1-line edit in cpl-check, test build        1.64    1.34    -18%
1-line edit in cpl-ir,    test build        1.51    1.12    -26%
cargo test --workspace (warm)              95.61   91.51     -4%
cargo nt (nextest, warm)                   52.78   50.86
target/ size                              2.2 GB  757 MB    -66%
```

The large win is the test runner, not the profile: the same tests take
91.5 s under `cargo test --workspace` and 50.9 s under `cargo nt` (-44%),
because nextest runs test binaries in parallel instead of one after another.

On the Devin VMs `~/.cargo/config.toml` already set `line-tables-only`; the
earlier baseline there was 5.91 s clean build, 5.14 s clean test build,
1.42 s and 1.24 s incremental test builds for `cpl-check` and `cpl-ir`, and
99.3 s for `cargo test --workspace`.

## Slowest crates

From `cargo build --workspace --timings` on a clean target (before / after
config). The workspace has no third-party dependencies, so these are all ours:

1. `cpl-prover` (`crates/prover`, 7.3k lines): 2.78 s / 1.44 s, no deps, runs
   alongside everything else.
2. `cpl-ir` (17.5k lines): 2.70 s / 2.09 s, and on the critical path: it
   starts only when `cpl-check` metadata is ready and gates `cpl-codegen`,
   `cpl-interp`, `cpl-stress`, `cpl-lsp`, `cpl-wasm` and `cpl-cli`.
3. `cpl-verify` (5.1k lines): 2.53 s / 1.13 s, no deps.
4. `cpl-check` (11.0k lines): 2.33 s / 1.44 s, critical path.
5. `cpl-syntax` (6.6k lines): 1.58 s, critical path, rebuilds everything.

After a one-line edit in `cpl-ir` the whole incremental build is 0.85 s,
spread over `cpl-ir` (0.34 s) and its six dependents.

The slowest test binaries in `cargo test --workspace` are dominated by `g++`,
not Rust: `cpl-verify/tests/proto_fuzz.rs` 34.1 s, one crate's unit-test binary
15.5 s, `cpl-cli/tests/differential.rs` 8.7 s, `cpl-verify/tests/proto_tools.rs`
7.8 s, `cpl-lsp/tests/latency.rs` 6.9 s.

## Next cycle

1. **Run the fast gate on nextest.** Make `scripts/check.sh --fast` call
   `cargo nextest run --workspace` when `cargo-nextest` is on `PATH` and fall
   back to `cargo test --workspace` otherwise. Measured: 91.5 s to 50.9 s per
   pre-commit run with the same tests.
2. **Take the `proto/cpc` fuzzer out of the fast gate.** `proto_fuzz.rs` and
   `proto_tools.rs` test the retired Python prototype and cost 41.9 s of the
   suite. Give them `required-features = ["slow"]` as `[[test]]` entries in
   `crates/cpl-verify/Cargo.toml` and run them from `scripts/check.sh` without
   `--fast` only.
3. **Split `cpl-ir` into the IR types and the lowering pass.** `lib.rs` and
   `show.rs` (about 1k lines) hold everything `cpl-codegen` and `cpl-interp`
   import; `lower/` (16.5k lines, `call.rs` alone 4.5k) is the slow part. A
   `cpl-lower` crate depending on `cpl-ir` would let codegen and interp compile
   in parallel with lowering on a clean build, and an edit to lowering would
   stop recompiling `cpl-codegen` and `cpl-interp` at all.

## Commands

```
cargo build --workspace --timings          # report in target/cargo-timings/
cargo test --workspace --no-run            # test build only
cargo nt                                   # whole suite on nextest
```
