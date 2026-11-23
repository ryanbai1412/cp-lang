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

Each row is one quality a user can feel in std, the metric, and the strongest
mechanism that enforces it. `missing` means nothing checks it yet. The rows
come from walking every journey in [journeys.md](../product/journeys.md) as
each user (contestant, learner, editor user, library author, contributor) and
from how mature libraries measure quality: the Rust API guidelines checklist,
Python's doctests, AtCoder Library's tests against brute force, LLVM's fuzzers
and `cargo-semver-checks`. Targets and today's numbers are in
`plan/domains/stdlib/metrics.toml`. Rows are built in order of user impact.

### Correct results (J5, J6, J9)

| Quality | Metric | Mechanism |
|---|---|---|
| Correct on hand-picked cases | std conformance cases failing | hand-written suite `spec/conformance/std-*/` in `cpl-conform all --check` |
| Correct on every input, not just picked ones | containers and algorithms with no random test against a brute-force model (`SortedBag` against a sorted `List`, `Fenwick` against prefix sums, `dijkstra` against Bellman-Ford) | `missing` (model-based property suite, seeded and shrinking, run on both engines) |
| Correct at the edges: empty, one element, all equal, max n, max value | public fns with no case for each edge their doc names (`empty:`, `ties:`, `traps:`) | `missing` (generated edge-case suite driven by the doc labels) |
| Algebraic laws hold | ops in `std/algebra.cp` whose laws are unproved | `crates/cpl-prover/src/laws.rs` |
| No overflow inside std for inputs in the declared range | std bodies with an unproved intermediate | `missing` (overflow-proof report over std bodies) |
| Traps fire exactly where documented | `traps:` lines with no trapping case | `missing` (generated trap suite from `traps:` labels) |
| Interpreter and C++ agree | `cpl-conform diff` disagreements on std | differential suite `cpl-conform diff`; side-effect variants in `crates/cpl-conform/src/variants.rs` |
| Prints, walks and hash orders are deterministic | print-order disagreements between engines | `cpl-conform diff` side-effect variants |
| Every trait method of every claimer works | uncovered (type, trait, method) triples | generated suite `scripts/conformance-traits.py --check`, gap list only shrinks |
| No undefined behaviour in emitted std code | sanitizer failures over std cases | sanitized C++ in `crates/cpl-cli/tests/differential.rs`; `missing` for the full std suite |
| A bug that escapes is caught next time | escaped std bugs with no row and no case | process: each escape adds its case and names the row that should have caught it |
| Tests catch real bugs | mutation score of the std suites | `missing` (mutation runs over `std/*.cp`, graded by tech-debt) |

### Guessable and consistent API (J2, J5)

| Quality | Metric | Mechanism |
|---|---|---|
| One spelling per idea | banned spellings in std and docs | `scripts/spec-vocab.py` |
| Names follow one lexicon (`first_ge`, `count`, `len`, `try_`, `_at`) | signatures that break `guidelines/std-api.md` | `missing` (API-consistency lint over `cpl doc --format json std`, shrink-only) |
| Searches and min/max return `?T`, never a sentinel | fns returning `-1`, `n` or `INF` for "none" | `missing` (same lint) |
| Options are named arguments (`by:`), never `_by` variants | `_by`/`_key` names, positional options | `missing` (same lint) |
| One form per operation: method or free fn, not both by accident | operations reachable both ways without a documented reason | `missing` (same lint) |
| Same argument order across neighbours | neighbour pairs with swapped order | `missing` (same lint) |
| Keys and indices keep their index space (positions, cuts, vertices, edges) | fns that take or return a bare `Int` where a domain exists | `missing` (same lint) |
| Every container has the common traits (`Show`, `Eq`, `Seq`, `len`, `is_empty`) | (type, expected trait) pairs missing | `missing` (trait matrix test over std types) |
| Spec and std agree on names and named arguments | unmatched `f(k: v)` in spec blocks | `scripts/conformance-sigs.py --check`, gap list only shrinks |
| Each type's common tasks are one short call | journeys per type with a tested example (`SortedBag`: insert, erase one, erase at cursor, predecessor, successor, range) | `missing` (usage-journey suite, first for `SortedBag`/`SortedSet`) |
| Composes: views, keys and ops pass through each other | pairs of std features that fail together (`.rev` into `zip`, `Window` into `maximal`) | `missing` (pairwise composition suite) |
| The obvious code is the fast code | std calls that look O(1) and are not | `missing` (scaling test, below) |

### Discoverable (J2, J5)

| Quality | Metric | Mechanism |
|---|---|---|
| Any value a program gets, it can name without a `use` | std types named by prelude signatures but not exported | `missing` (signature-walk test over `std/prelude.cp`) |
| Spec tables and the prelude agree | missing prelude names | `scripts/conformance-prelude.py --check`; `scripts/spec-stub.py --check` the other way |
| Findable without knowing the name (`lower_bound` finds `first_ge`) | common C++/Python names with no alias line in docs | `missing` (alias table checked against the docs index) |
| Hover and completion show the full signature, options and bounds | std fns whose `cpl doc --format json` lacks bounds or defaults | `missing` (doc JSON shape test) |
| Go-to-definition opens readable std source | std files over the size cap, fns with no doc | `scripts/debt-ratchet.py` caps; `scripts/spec-stub.py --audit` |

### Documented truthfully (J2, J5, J8)

| Quality | Metric | Mechanism |
|---|---|---|
| Every doc example compiles and runs | failing `example:` lines | `cpl-conform docs`, ratcheted by the scoreboard; a hard gate per example is `missing` |
| Every spec/lib example compiles and runs | failing code blocks in `spec/lib/*.md` | `crates/cpl-conform/src/spec_blocks.rs` in `cpl-conform all --check` |
| Every public fn states cost, traps and edge cases | fns missing a required label | `scripts/spec-stub.py --audit` |
| The documented cost is the real cost | fns slower than their `cost:` | `missing` (scaling test per fn: time at n and 4n) |
| Docs show trait bounds and required labels | rendered docs missing bounds | `missing` (doc JSON shape test) |
| Docs are readable: summary first, short, example near the top | pages over the length limit, summaries over one line | `missing` (doc readability lint) |
| A person or model writes correct code from the docs alone | unseen problems solved first try from docs only | process: cold-reader rounds, each miss traced to a row here |

### Fast and small (J8, J9)

| Quality | Metric | Mechanism |
|---|---|---|
| Within 1.1x of matched C++ | p50 and max ratio over std benches | `cpl bench` matched baselines in `bench/`, owned with perf |
| No hidden allocation or copy | std fns that allocate without saying so | `missing` (allocation counter in the interpreter checked against docs) |
| Small submissions | bytes of std code per emitted solution | `crates/cpl-cli/tests/read_only_copies.rs` counts std fns; a size budget is `missing` |
| Fast front end with std loaded | check time of an empty program | `missing` (timing test, budget under 500 ms) |
| Compiles fast on the judge | g++ time of std-heavy solutions | `missing` |

### Debuggable (J6, J7)

| Quality | Metric | Mechanism |
|---|---|---|
| Errors from a std call point at the user's call, never into std | diagnostics whose primary span is in `std/` | `missing` (mutation suite of wrong calls checked for span location) |
| Trap messages name the user's call and values | trap messages that show std internals | `missing` (same suite, traps half) |
| Traces show std values by their meaning, not their fields | std types whose trace shows internals | `crates/cpl-cli/tests/trace_goldens.rs` per program; a check per std type is `missing` |
| Stress can generate every std input type | std types with no generator | `missing` |

### No privilege and stable (J10)

| Quality | Metric | Mechanism |
|---|---|---|
| std uses only powers user libraries have | std names special-cased in the compiler | `plan/domains/stdlib/measure.py privilege`, ratcheted with tech-debt |
| A std module copied into a user library runs the same | modules that fail when copied | `missing` (copy-a-module test) |
| std passes the rules a user library meets | `cpl check std/<m>.cp` errors per file | `crates/cpl-cli/tests/std_checks.rs`, each count only drops |
| No unimplemented bodies | `todo` stubs | `scripts/debt-ratchet.py` (`stdlib_todo`) |
| Modules import without cycles | explicit import cycles | `measure.py cycles`; a gate test is `missing` |
| Public API changes are never silent | signature changes with no codemod or release note | `missing` (public-signature snapshot test, like `cargo-semver-checks`) |
| Old solutions keep building | corpus ports broken by a std change | corpus in `cpl-conform all --check` |
| Lint-clean std | lint hits in std (`L0301` float equality and others) | `cpl-lint` baselines, shrink-only; each kept hit is marked `with lint(...: allow)` |

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
- 2026-11-23: register made exhaustive across journeys and users.
