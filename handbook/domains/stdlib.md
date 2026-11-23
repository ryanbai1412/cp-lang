# stdlib

| | |
|---|---|
| Kind | Domain charter |
| Owner | stdlib |
| Describes | `std/*.cp`, `std/prelude.cp`, `std/README.md`, `std/GAPS.md`, `spec/conformance/std-*/`, `plan/domains/stdlib/` |

## Mission

Every contest facility a contestant reaches for is one guessable call, with
its cost, traps and a runnable example, written in plain CP Lang that a user
library could have written too. Example: a contestant who has never read the
docs for `std.bag` should guess `bag.insert(x)`, `bag.remove(x)` and
`bag.first_ge(x)`, and each should do what its name says in the cost its doc
states.

## Scope

Owned:

- Every module `std/<m>.cp`: signatures, `///` docs (`cost:`, `traps:`,
  `example:`, `empty:`, `ties:`, `order:`), bodies, in-file `@test` fns.
- `std/prelude.cp`, the one list of names every program gets without a `use`.
- `std/GAPS.md`: each body the compiler can't run yet, with its compiler request.
- The `spec/conformance/std-*/` cases (stdlib writes them; spec owns the
  harness rules).
- Handbook docs: `guidelines/std-api.md`, `guidelines/std-docs.md`,
  `design/std-mechanisms.md`, `language/std.md`.
- `plan/domains/stdlib/` (plan, metrics, worklog, `measure.py`).

Not owned:

| Not here | Owner |
|---|---|
| The library chapters `spec/lib/*.md` | spec |
| How the compiler lowers calls, builtins and intrinsics (`crates/cpl-ir`, `crates/cpl-check`) | compiler |
| Removing compiler privilege as a ratchet | tech-debt (stdlib does the std side) |
| C++ speed of std bodies, `cpl bench` | perf (stdlib supplies matched benches) |
| `cpl doc`, hover and completion over std | tools |
| Site reference pages generated from std | site |
| `cpl-conform` itself | infra, spec |

## Interfaces

Provides:

| What | Where | Used by |
|---|---|---|
| Module source, bundled into every `cpl` binary | `crates/cpl-check/build.rs`, `crates/cpl-check/src/modules.rs` | every command, LSP, playground |
| The prelude list | `std/prelude.cp`, read by `crates/cpl-check/src/prelude.rs` | checker, LSP completion, lint `redundant_prelude_import` |
| Doc comments | `cpl doc --format json std` | site reference, LSP hover, `scripts/spec-stub.py` |
| `/// example:` lines as tests | `crates/cpl-conform/src/docs.rs` | `cpl-conform docs`, the scoreboard |
| Algebra laws | `std/algebra.cp` | `crates/cpl-prover/src/laws.rs` |

Needs:

| What | From |
|---|---|
| Every power std uses is open to user libraries: `impl Index` domains, `@get`, `lends`, generic traits, `each` walks | compiler |
| Method lookup by self type, inference through trait bounds | compiler |
| A matched C++ baseline per hot std fn | perf |
| Hover, completion and go-to-definition that read std's own signatures and docs | tools |

## Quality register

Each row is one quality std must be high on, the metric, and the strongest
mechanism that enforces it. `missing` means nothing checks it yet. Targets and
today's numbers are in `plan/domains/stdlib/metrics.toml`.

| Quality | Metric | Mechanism |
|---|---|---|
| Correct bodies | std conformance cases passing | hand-written suite `spec/conformance/std-*/` in `cpl-conform all --check` |
| Interpreter and C++ agree | `cpl-conform diff std-` fails and skips | differential suite `cpl-conform diff`; side-effect variants in `crates/cpl-conform/src/variants.rs` |
| Every trait method of every claimer works | uncovered (type, trait, method) triples | generated suite `scripts/conformance-traits.py --check`, gap list only shrinks |
| Algebraic laws hold (associativity, identity) | ops whose laws are unproved | `crates/cpl-prover/src/laws.rs` over `std/algebra.cp` |
| Every doc example compiles and runs | failing `example:` lines | `cpl-conform docs`, ratcheted by the scoreboard; a hard gate per new example is `missing` |
| Every spec/lib example compiles and runs | failing code blocks in `spec/lib/*.md` | `crates/cpl-conform/src/spec_blocks.rs` in `cpl-conform all --check` |
| Spec and std agree on named arguments | unmatched `f(k: v)` in spec blocks | `scripts/conformance-sigs.py --check`, gap list only shrinks |
| Spec tables and the prelude agree | missing prelude names | `scripts/conformance-prelude.py --check`; `scripts/spec-stub.py --check` the other way |
| Any value a program gets, it can annotate without a `use` | std types named by prelude signatures but not exported | `missing` (signature-walk test over `std/prelude.cp`) |
| Every public fn says its cost, traps and edge cases | fns missing a required doc label | `scripts/spec-stub.py --audit` |
| The documented cost is the real cost | bodies slower than their `cost:` | `missing` (scaling test per fn: time at n and 4n) |
| One spelling per idea | banned spellings | `scripts/spec-vocab.py` |
| Guessable, consistent API: `?T` for searches and min/max, `by:` options, one form per operation | violations of `guidelines/std-api.md` | `missing` (API-consistency lint over std signatures, as a shrink-only ratchet) |
| Each type's common tasks are one short call | journeys per type with a tested example | `missing` (usage journeys, first for `SortedBag`/`SortedSet`) |
| No stdlib privilege | std names special-cased in the compiler | `plan/domains/stdlib/measure.py privilege`, ratcheted with tech-debt |
| A std module copied into a user library runs the same | modules that fail when copied | `missing` (copy-a-module test) |
| std passes the rules a user library meets | `cpl check std/<m>.cp` errors per file | `crates/cpl-cli/tests/std_checks.rs`, each count only drops |
| No unimplemented bodies | `todo` stubs | `scripts/debt-ratchet.py` (`stdlib_todo`), only drops |
| Modules import each other without cycles | explicit import cycles | `measure.py cycles`; a gate test is `missing` |
| Exact float comparisons only where meant | `float_equality` hits in std | lint `L0301` baseline; each kept one is marked `with lint(float_equality: allow)` |
| Prints are the same on both engines | print-order disagreements | `cpl-conform diff` side-effect variants |
| Writable from the docs alone | unseen problems solved first try from the docs | process: cold-contestant runs by the idioms domain, each miss traced to a row above |

## Suites and tools

| Tool | Catches | Run |
|---|---|---|
| `spec/conformance/std-*/` | wrong results, wrong traps | `target/release/cpl-conform conformance std-<m>` |
| differential | interpreter and C++ disagreeing | `target/release/cpl-conform diff std-<m>` |
| doc examples | `example:` lines that fail | `target/release/cpl-conform docs <m>.cp` (the filter is a substring, keep `.cp`) |
| `cpl-conform all --check` | any suite dropping below the scoreboard | `scripts/check.sh` |
| `scripts/conformance-traits.py` | trait methods with no case | `uv run scripts/conformance-traits.py --check` |
| `scripts/conformance-sigs.py` | spec named arguments std lacks | `uv run scripts/conformance-sigs.py --check` |
| `scripts/conformance-prelude.py` | spec table names missing from the prelude | `uv run scripts/conformance-prelude.py --check` |
| `scripts/spec-stub.py` | missing doc labels, `@get` members that do work | `python3 scripts/spec-stub.py --check`, `--audit` |
| `scripts/spec-vocab.py` | banned spellings | `python3 scripts/spec-vocab.py` |
| `std_checks` | std breaking user-library rules | `cargo test --release -p cpl-cli --test std_checks` |
| `scripts/debt-ratchet.py` | `todo` count growing | `python3 scripts/debt-ratchet.py --check` |
| `measure.py` | privilege, cycles, uncovered modules, `todo` split | `python3 plan/domains/stdlib/measure.py <metric>` |

Land with `CPL_LANE=stdlib scripts/land.sh`.

## Changes

- 2026-11-23: first charter, from `plan/domains/stdlib/README.md` and `ARCHITECTURE.md`.
