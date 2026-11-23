# stdlib

| | |
|---|---|
| Kind | Domain charter |
| Owner | stdlib |
| Describes | `std/*.cp`, `std/prelude.cp`, `std/README.md`, `std/GAPS.md`, `spec/conformance/std-*/`, `plan/domains/stdlib/` |

The standard library (std) is the CP Lang code every program can call:
containers, range structures, graph and string algorithms, math, and the
prelude of names available without a `use`. It is written in plain CP Lang,
so anything std does, a contestant's own library can do too. If you have
called `a.sort()`, `dijkstra(...)` or `SegTree(...)`, you have used it.

## What good looks like

A contestant who has never read std's docs reaches for a facility and
guesses its call. They type `bag.add(x)` and `bag.remove(x)` on a
`SortedBag`, `a.scan(+)` for prefix sums, `xs.tally()` to count values,
and each does what its name says, at the cost its hover shows. Searching
an empty list gives `None`, not `-1`. When they do read a doc, the first
line says what the fn is for, the next lines give its cost, edge cases and
traps, and a one-line example they can copy runs as written.

## How it fits together

- **Modules** in `std/<m>.cp` hold the signatures, the `///` docs and the
  bodies. `std/prelude.cp` is the one list of names every program gets.
- **Every `cpl` binary embeds std**, so the checker, interpreter, C++
  backend, language server and playground all see the same source.
- **std uses only ordinary powers:** `impl Index` for library index
  spaces, `@get` members, generic traits, `each` walks.
  [std-mechanisms.md](../design/std-mechanisms.md) explains each, and
  [language/std.md](../language/std.md) explains how programs see std.
- **Two rule pairs keep it consistent:** [std-api.md](../guidelines/std-api.md)
  with its [checklist](../guidelines/std-api-checklist.md) for signatures,
  and [std-docs.md](../guidelines/std-docs.md) with its
  [checklist](../guidelines/std-docs-checklist.md) for doc comments.

## How we know it is good

Hand-written conformance cases (`spec/conformance/std-*/`) check results
and traps on both engines, the differential suite checks that the
interpreter and the emitted C++ agree, every `example:` line in a doc runs
as a test, and scripts check that the spec, the prelude and std agree on
names and options.

The weakest areas today: no container is tested against a brute-force
model on random inputs, no lint checks signatures against the naming
rules, the everyday tasks of each type have no usage suite, failing doc
examples are counted rather than blocking, and errors from a bad std call
can still point into std. The quality register below lists every quality
and its check; rows marked `missing` are built in order of user impact.

## Where to start

- **Adding or changing a std fn:** read [std-api.md](../guidelines/std-api.md),
  edit `std/<m>.cp` with a doc that meets the docs checklist, and add a case
  under `spec/conformance/std-<m>/`.
- **Fixing a wrong result:** reproduce it as a conformance case first, then
  run `target/release/cpl-conform conformance std-<m>` and
  `target/release/cpl-conform diff std-<m>`.
- **Checking docs:** `target/release/cpl-conform docs <m>.cp` and
  `python3 scripts/spec-stub.py --audit`.

---

# Reference

The rest of this document is for looking things up: the exact scope, the
interfaces other domains rely on, every quality we track and the suites
that check them.

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

Decided with Ryan, through `plan/QUESTIONS.md`: API shape, naming, and what
belongs in std and in the prelude, judged on real ports. Everything else is
decided in this domain and logged in `decisions.md`.

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

### Modules

Each module is one file, and its opening `//!` line is the source of truth
for its purpose; this table copies those lines.

| Module | Purpose |
|---|---|
| `algebra` | Operations as values: what a range structure folds with. |
| `bag` | Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`. |
| `beats` | Segment Tree Beats (Ji Driver) for range chmin, chmax, add, sum, min, and max. |
| `bitset` | A dense set of positions of a domain, 64 per word. |
| `deque` | Double-ended queue: `Deque<T>`. |
| `derive` | The `Eq`, `Ord`, `Show` and `Hash` a struct gets field by field, as library code on |
| `dp` | The DP toolkit: the structures and optimizations a DP calls by name. |
| `dsu` | Disjoint sets over a domain, with optional values per set, parities, and rollback. |
| `fenwick` | Fenwick tree (binary indexed tree) over a domain: point add, range sum. |
| `flow` | Flows, cuts and matchings: Dinic's max flow and min cut, min-cost flow, maximum |
| `geometry` | Plane and 3D geometry on exact integer points (`Pt`, `Pt3`) and careful |
| `graph` | Graphs: one stored representation and the algorithms over "what comes after v". |
| `grid` | Grids: a cell is the pair `(Row, Col)` and there is no grid type. `[Row, Col]T` is the grid, |
| `hashmap` | Open-addressing hash map for `Int` keys with splitmix64 salting. |
| `heap` | Binary heaps (round 5): `Heap<T>(by:)` pops the smallest key first and `Heap(by: desc)` the |
| `interval` | Interval partition map (ODT / Chtholly tree): `IntervalMap<D, T>`. |
| `io` | Answer words for yes/no problems. `print`, `read` and the `io` context are core |
| `lazy` | Range-update helpers that are not segment trees |
| `marks` | Opt-in generation-stamped mark table: `add`, `in`, `remove`, and `clear` are all \(O(1)\). |
| `math` | Number theory on plain `Int`s. |
| `matrix` | Dense matrices, exact and floating-point Gaussian elimination, and GF(2) XOR bases. |
| `mo` | Mo's algorithm: offline ordering and window stepping over `Range<D>`. |
| `mod` | Modular arithmetic and counting. |
| `ntt` | Number-theoretic transform, exact and modular convolution, SOS, and bitwise transforms. |
| `opt` | Methods of optionals (`?T`, core 2.4). The operators (`??`, `!`, |
| `pair` | Unordered pairs of distinct positions: the domain of data given once per pair. |
| `persistent` | Persistent segment tree and static range k-th smallest queries. |
| `poly` | Formal power series operations over NTT-friendly `Mod<P>`. |
| `prefix` | Prefix sums, prefix/suffix monoid folds, and offline difference arrays. |
| `prelude` | The prelude: the names of the standard library every program has |
| `rand` | Random numbers from a seed: the same seed gives the same numbers in `cpl run` |
| `search` | Search on answers: binary, exponential, ternary and parallel searches, |
| `segtree` | Segment tree over a domain, for any associative operation. |
| `seq` | Sequences: reductions, scans, runs, searching, permutations, monotone stacks, and compression. |
| `slope` | Integer-indexed concave and convex functions as sorted slopes (slope trick). |
| `sparse` | Sparse table: \(O(1)\) range folds of a fixed array. |
| `spec` | Checking observations on small cases: is a table monotone, convex, unimodal or Monge, does a |
| `strings` | String algorithms. They only compare elements, so they work on a `Str` |
| `subset` | Subsets of a small domain as one machine word: the mask of mask DP with its domain kept. |
| `timetree` | Offline segment tree over time (divide-and-conquer on life intervals). |
| `tree` | Rooted trees: parents, subtree ranges, LCA, and path folds over one range structure. |
| `window` | Sliding-window folds and stateful windows (round 7, `lib/sequence-patterns.md`). |

## Interfaces

Provides:

| What | Where | Used by |
|---|---|---|
| Module source, bundled into every `cpl` binary | `crates/cpl-check/build.rs`, `crates/cpl-check/src/modules.rs` | every command, LSP, playground |
| The prelude list | `std/prelude.cp`, read by `crates/cpl-check/src/prelude.rs` | checker, LSP completion, lint `redundant_prelude_import` |
| Doc comments | `cpl doc --format json std` | site reference, LSP hover, `scripts/spec-stub.py` |
| `/// example:` lines as tests | `crates/cpl-conform/src/docs.rs` | `cpl-conform docs`, the scoreboard |
| Algebra laws | `std/algebra.cp` | `crates/cpl-prover/src/laws.rs` |
| Names the compiler binds to std (`BUILT_ON`, `INTERNAL_BUILT_ON`, `std_module`) | `crates/cpl-check/src/prelude.rs` | checker, `cpl fix` import edits |
| Module names for `use std.` completion | `crates/cpl-lsp/build.rs` | LSP |
| Conformance cases, one directory per module | `spec/conformance/std-<m>/` | `cpl-conform conformance`, `cpl-conform diff` |
| Modules that pass a standalone `cpl check` | `SPEC_STD` in `crates/cpl-cli/tests/corpus.rs` | `cargo test -p cpl-cli` |

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
| Correct at the edges: empty, one element, all equal, max n, max value | public fns with no case for each edge their doc names (`empty:`, `ties:`, `traps:`) | rule `std-docs/R2`; `missing` (generated edge-case suite driven by the doc labels) |
| Algebraic laws hold | ops in `std/algebra.cp` whose laws are unproved | `crates/cpl-prover/src/laws.rs` |
| A fold of a monoid is `T`, and any op works with a `?T` fold | std folds or `Window.all` returning `?T` for a monoid; structures rejecting an op with no identity | rule `std-api/R5`; `Monoid.of` examples in `cpl-conform docs`; `missing` for `Window.all` and the `?T` lifting helper |
| No overflow inside std for inputs in the declared range | std bodies with an unproved intermediate | `missing` (overflow-proof report over std bodies) |
| Traps fire exactly where documented | `traps:` lines with no `trap:` example (`--audit`) | rule `std-docs/R4`; `scripts/std-trap-examples.py` (in `scripts/check.sh`) |
| Interpreter and C++ agree | `cpl-conform diff` disagreements on std | differential suite `cpl-conform diff`; side-effect variants in `crates/cpl-conform/src/variants.rs` |
| Prints, walks and hash orders are deterministic | print-order disagreements between engines | `cpl-conform diff` side-effect variants |
| Every trait method of every claimer works | uncovered (type, trait, method) triples | generated suite `scripts/conformance-traits.py --check`, gap list only shrinks |
| No undefined behaviour in emitted std code | sanitizer failures over std cases | sanitized C++ in `crates/cpl-cli/tests/differential.rs`; `missing` for the full std suite |
| A bug that escapes is caught next time | escaped std bugs with no row and no case | process: each escape adds its case and names the row that should have caught it |
| Tests catch real bugs | mutation score of the std suites | `missing` (mutation runs over `std/*.cp`, graded by tech-debt) |

### Guessable and consistent API (J2, J5)

| Quality | Metric | Mechanism |
|---|---|---|
| One spelling per idea | banned spellings in std and docs | rule `std-api/R1`; `scripts/spec-vocab.py` |
| Names follow one lexicon (`first_ge`, `count`, `len`, `try_`, `_at`) | signatures that break `guidelines/std-api.md` | rule `std-api/R1`; `missing` (API-consistency lint over `cpl doc --format json std`, shrink-only) |
| Searches and min/max return `?T`, never a sentinel | fns returning `-1`, `n` or `INF` for "none" | rule `std-api/R4`; `scripts/std-api-lint.py` (in `scripts/check.sh`, shrink-only) |
| Options are named arguments (`by:`), never `_by` variants | `_by`/`_key` names, positional options | rule `std-api/R6`; `scripts/std-api-lint.py` (in `scripts/check.sh`, shrink-only) |
| One form per operation: method or free fn, not both by accident | operations reachable both ways without a documented reason | rule `std-api/R11`; `missing` (same lint) |
| Same argument order across neighbours | neighbour pairs with swapped order | rule `std-api/R8`; `missing` (same lint) |
| Keys and indices keep their index space (positions, cuts, vertices, edges) | fns that take or return a bare `Int` where a domain exists | rule `std-api/R9`; `missing` (same lint) |
| Every container has the common traits (`Show`, `Eq`, `Seq`, `len`, `is_empty`) | (type, expected trait) pairs missing | rule `std-api/R12`; `missing` (trait matrix test over std types) |
| Spec and std agree on names and named arguments | unmatched `f(k: v)` in spec blocks | rule `std-api/R6`; `scripts/conformance-sigs.py --check`, gap list only shrinks |
| Each type's common tasks are one short call | journeys per type with a tested example (`SortedBag`: insert, erase one, erase at cursor, predecessor, successor, range) | rule `std-api/R13`; `missing` (usage-journey suite, first for `SortedBag`/`SortedSet`) |
| Composes: views, keys and ops pass through each other | pairs of std features that fail together (`.rev` into `zip`, `Window` into `maximal`) | rule `std-api/R14`; `missing` (pairwise composition suite) |
| Every common operation from the C++ STL, Python, Rust and the AtCoder Library has a spelling | inventory operations with no spelling and no listed gap | rule `std-api/R15`; `missing` (comparative inventory checked against `cpl doc --format json std`) |
| The obvious code is the fast code | std calls that look O(1) and are not | rule `std-api/R16`; `missing` (scaling test, below) |

### Discoverable (J2, J5)

| Quality | Metric | Mechanism |
|---|---|---|
| Any value a program gets, it can name without a `use` | std types named by prelude signatures but not exported | rule `std-api/R18`; `scripts/prelude-signatures.py` (in `scripts/check.sh`) |
| Spec tables and the prelude agree | missing prelude names | `scripts/conformance-prelude.py --check`; `scripts/spec-stub.py --check` the other way |
| Findable without knowing the name (`lower_bound` finds `first_ge`) | common C++/Python names with no alias line in docs | `missing` (alias table checked against the docs index) |
| Hover and completion show the full signature, options and bounds | std fns whose `cpl doc --format json` lacks bounds or defaults | rule `std-docs/R5`; `missing` (doc JSON shape test) |
| Go-to-definition opens readable std source | std files over the size cap, fns with no doc | `scripts/debt-ratchet.py` caps; `scripts/spec-stub.py --audit` |

### Documented truthfully (J2, J5, J8)

| Quality | Metric | Mechanism |
|---|---|---|
| Every doc example compiles and runs | failing `example:` lines | rule `std-docs/R6`; `cpl-conform docs`, ratcheted by the scoreboard; a hard gate per example is `missing` |
| Every spec/lib example compiles and runs | failing code blocks in `spec/lib/*.md` | `crates/cpl-conform/src/spec_blocks.rs` in `cpl-conform all --check` |
| Every public fn states cost, traps and edge cases | fns missing a required label | rule `std-docs/R1`; `scripts/spec-stub.py --audit` |
| The documented cost is the real cost | fns slower than their `cost:` | rule `std-docs/R3`; `missing` (scaling test per fn: time at n and 4n) |
| Docs show trait bounds and required labels | rendered docs missing bounds | rule `std-docs/R5`; `missing` (doc JSON shape test) |
| Docs are readable: summary first, short, example near the top | pages over the length limit, summaries over one line | rule `std-docs/R8`; `missing` (doc readability lint) |
| A person or model writes correct code from the docs alone | unseen problems solved first try from docs only | process: cold-reader rounds, each miss traced to a row here |
| Every std page is verified against the docs checklist | modules not verified in `plan/domains/stdlib/docs-tracker.md` | rule `std-docs/R11`; process, the verified count only goes up |

### Fast and small (J8, J9)

| Quality | Metric | Mechanism |
|---|---|---|
| Within 1.1x of matched C++ | p50 and max ratio over std benches | `cpl bench` matched baselines in `bench/`, owned with perf |
| No hidden allocation or copy | std fns that allocate without saying so | `missing` (allocation counter in the interpreter checked against docs) |
| Fixed-size `windows(k)` for a compile-time `k` does not allocate | allocations per window | rule `std-api/R17`; `missing` (allocation counter in the interpreter) |
| Small submissions | bytes of std code per emitted solution | `crates/cpl-cli/tests/read_only_copies.rs` counts std fns; a size budget is `missing` |
| Fast front end with std loaded | check time of an empty program | `missing` (timing test, budget under 500 ms) |
| Compiles fast on the judge | g++ time of std-heavy solutions | `missing` |

### Debuggable (J6, J7)

| Quality | Metric | Mechanism |
|---|---|---|
| Errors from a std call point at the user's call, never into std | diagnostics whose primary span is in `std/` | rule `errors/R3`; `missing` (mutation suite of wrong calls checked for span location) |
| Trap messages name the user's call and values | trap messages that show std internals | `missing` (same suite, traps half) |
| Traces show std values by their meaning, not their fields | std types whose trace shows internals | `crates/cpl-cli/tests/trace_goldens.rs` per program; a check per std type is `missing` |
| Stress can generate every std input type | std types with no generator | `missing` |

### No privilege and stable (J10)

| Quality | Metric | Mechanism |
|---|---|---|
| std uses only powers user libraries have | std names special-cased in the compiler | rule `std-api/R19`; `plan/domains/stdlib/measure.py privilege`, ratcheted with tech-debt |
| A std module copied into a user library runs the same | modules that fail when copied | `missing` (copy-a-module test) |
| std passes the rules a user library meets | `cpl check std/<m>.cp` errors per file | `crates/cpl-cli/tests/std_checks.rs`, each count only drops |
| No unimplemented bodies | `todo` stubs | `scripts/debt-ratchet.py` (`stdlib_todo`) |
| Modules import without cycles | explicit import cycles | `crates/cpl-check/tests/std_acyclic.rs`; `measure.py cycles` for the graph |
| Public API changes are never silent | signature changes with no codemod or release note | rule `std-api/R20`; `missing` (public-signature snapshot test, like `cargo-semver-checks`) |
| Old solutions keep building | corpus ports broken by a std change | corpus in `cpl-conform all --check` |
| Lint-clean std | lint hits in std (`L0301` float equality and others) | `cpl-lint` baselines, shrink-only; each kept hit is marked `with lint(...: allow)` |

## Review rubric

A reviewer who has not seen the module samples 10 public fns and scores each
1 to 5 on five questions, with a reason. The trend matters more than any one
score.

1. **Guessable.** Knowing only its neighbours, would you have guessed this
   name, its argument order and its options? (`std-api/R1`, `std-api/R8`)
2. **Cost honest.** Does the body do what `cost:` says, and does anything
   O(n) or allocating look like it? (`std-api/R16`, `std-docs/R3`)
3. **One way.** Is this the only std call for its job? (`std-api/R11`)
4. **Teaches.** Would a contestant learn the algorithm from the body, as from
   KACTL or ACL?
5. **No privilege.** Could a user library have written this module?
   (`std-api/R19`)

Probes, run by a reader with only the docs (pass or fail, time, files touched):

- **Write from the docs.** Solve 5 unseen problems with std calls that
  compile and run first try.
- **Add a std fn.** Follow `std-api` and `std-docs`; the gate is green first
  try and at most 3 files change.
- **Copy a module.** Copy `std.fenwick` into a user library and run its cases
  from there.
- **Audit costs.** Given 10 bodies, find the one with a planted wrong `cost:`.

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
- 2026-11-23: overview first; register rows cite the std-api, std-docs and errors checklists.
- 2026-11-23: module map, review rubric and probes moved in from the plan folder.
