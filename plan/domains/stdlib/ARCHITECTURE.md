# stdlib architecture

The code as it is on main `4b8f504f` (2026-11-22). Plan and targets: [PLAN.md](PLAN.md). Normative rules for std code: [std/README.md](../../../std/README.md).

## Shape

The standard library is 39 CP Lang modules in `std/*.cp` plus the prelude list `std/prelude.cp`: 40 files, 1,976 public items (`grep -cE '^\s*pub\s+(fn|struct|enum|trait|type)'`), 283 `@test` fns. A module is one file; `use std.<m>` names it. Bodies are CP Lang; a stub is `= todo` (294 today). `std/GAPS.md` lists what the compiler can't run yet.

Std modules import each other with explicit `use std.<m>` (57 edges, 0 cycles: `measure.py cycles`). Library modules never get the prelude (`std/prelude.cp` header); only the program being checked does.

## Modules

Lines and public items per file, purpose from each file's `//!` header.

| Module | Lines | Pub | Purpose |
|---|---|---|---|
| `algebra` | 526 | 46 | Operations as values (monoids, actions) that range structures fold with |
| `bag` | 2417 | 133 | `SortedBag<K>`, `SortedSet<K>`: ordered multiset/set, order statistics, cursors |
| `beats` | 629 | 22 | Segment tree beats: range chmin/chmax/add with sum/min/max |
| `bitset` | 1311 | 84 | Dense set of positions of a domain, 64 per word |
| `deque` | 603 | 33 | `Deque<T>` |
| `dp` | 1007 | 36 | DP toolkit: Li Chao (hull trick), divide-and-conquer layers, interval DP, and others a DP calls by name |
| `dsu` | 917 | 56 | Disjoint sets over a domain: values per set, parity, rollback |
| `fenwick` | 511 | 30 | Fenwick tree over a domain |
| `flow` | 1791 | 61 | Dinic max flow / min cut, min-cost flow, matchings |
| `geometry` | 3599 | 130 | Exact-integer plane and 3D geometry (`Pt`, `Pt3`), hulls, Delaunay |
| `graph` | 2905 | 122 | One stored graph representation and the algorithms over it |
| `grid` | 209 | 13 | Grids as `[Row, Col]T`; neighbours, no grid type |
| `hashmap` | 509 | 27 | Open-addressing `Int`-keyed hash map, splitmix64 salt |
| `heap` | 1177 | 69 | `Heap<T>(by:)`, indexed/meldable heaps |
| `interval` | 662 | 38 | `IntervalMap<D, T>` (ODT / Chtholly) |
| `io` | 70 | 5 | Answer words (`yes`/`no`); `print`/`read` are core |
| `lazy` | 724 | 28 | `Flip`, `LazyTreap`, `Rope`: range updates outside segment trees |
| `marks` | 516 | 36 | Generation-stamped mark table, O(1) clear |
| `math` | 1674 | 76 | Number theory on `Int` |
| `matrix` | 1259 | 62 | Dense matrices, Gaussian elimination, GF(2) XOR basis |
| `mo` | 317 | 6 | Mo's algorithm over `Range<D>` |
| `mod` | 527 | 29 | `Mod<P>` arithmetic and counting |
| `ntt` | 661 | 16 | NTT, convolutions, SOS and bitwise transforms |
| `opt` | 22 | 2 | Methods of `?T` (operators are core 2.4) |
| `pair` | 48 | 5 | Unordered pairs of distinct positions as a domain |
| `persistent` | 634 | 35 | Persistent segment tree, static range k-th |
| `poly` | 1220 | 53 | Formal power series over `Mod<P>` |
| `prefix` | 809 | 54 | Prefix sums/folds, difference arrays |
| `prelude` | 113 | 0 | The one list of prelude names, as `use` lines (192 names) |
| `rand` | 121 | 10 | Seeded random numbers, same in `cpl run` and C++ |
| `search` | 359 | 9 | Binary, exponential, ternary, parallel search on answers |
| `segtree` | 1132 | 51 | Segment tree over a domain, any associative op |
| `seq` | 3373 | 180 | Reductions, scans, runs, search, permutations, monotone stacks, compression |
| `slope` | 390 | 35 | Slope trick |
| `sparse` | 412 | 21 | Sparse table |
| `strings` | 3347 | 159 | String algorithms over any comparable sequence |
| `subset` | 155 | 9 | Subsets of a small domain as a word (mask DP) |
| `timetree` | 310 | 11 | Offline segment tree over time |
| `tree` | 2682 | 167 | Rooted trees: parents, subtree ranges, LCA, path folds |
| `window` | 310 | 17 | Sliding-window folds |

## Interfaces other domains use

| Interface | Where | Consumers |
|---|---|---|
| Module text bundled into the compiler (with `@test` fns stripped by `without_tests`) | `crates/cpl-check/build.rs`, `crates/cpl-check/src/modules.rs` (`bundled_std`, `std_modules`) | every `cpl` command, LSP, playground |
| The prelude list | `std/prelude.cp`, read via `crates/cpl-check/src/prelude.rs`; size gate in `scripts/check.sh` | checker, LSP completion, lint L0208 `redundant_prelude_import` (`crates/cpl-lint/src/lib.rs`), `crates/cpl-cli/tests/vscode_sync.rs` |
| Names the compiler binds to std (`BUILT_ON`, `INTERNAL_BUILT_ON`, `std_module`, `std_names`) | `crates/cpl-check/src/prelude.rs` | checker, `cpl fix` import edits |
| Module names for `use std.` completion | `crates/cpl-lsp/build.rs` | LSP |
| `///` docs (cost, traps, example, ...) | `std/README.md` §1 rules; `cpl doc --format json std` | `web/scripts/gen-ref.mjs` (site reference), LSP hover, `scripts/spec-stub.py` |
| `/// example:` lines as tests | `crates/cpl-conform/src/docs.rs` | `cpl-conform docs`, the scoreboard |
| Conformance cases | `spec/conformance/std-*/` (38 dirs, one per module except `opt`) | `cpl-conform conformance`, `diff` (interp vs C++) |
| Algebra laws | `std/algebra.cp` | `crates/cpl-prover/src/laws.rs` |
| Standalone-check ratchet | `SPEC_STD` in `crates/cpl-cli/tests/corpus.rs` (12 modules) | `cargo test -p cpl-cli` |
| `todo` count | `scripts/debt-ratchet.py` (`stdlib_todo`, baseline `scripts/debt-ratchet.json`) | `check.sh` |

## Invariants

- Std is ordinary CP Lang: a module uses public language features, documented intrinsics and other std modules, nothing else (`plan/ARCHITECTURE.md`). **Partly broken today**, see debt.
- One list of prelude names: `std/prelude.cp`; nothing else adds a name to every program.
- A library module gets no prelude; it `use`s what it needs.
- Std imports are acyclic (0 explicit cycles today; a gate test is on `devin/wip-std-import-cycles`).
- Every public fn carries `cost:`, `traps:`, `example:`, and `empty:` / `ties:` / `order:` where they apply (`std/README.md`); enforced by `spec-stub.py --audit` (32 missing).
- Every `example:` is a test; every module has `spec/conformance/std-<m>/` cases (all but `opt`).
- One spelling per idea: options are named arguments (`by:`), no `_by` twins (`spec-vocab.py`: 0 banned).
- `todo` only goes down (`debt-ratchet.py`).

## Known debt

- **Builtin shadowing / privilege.** `crates/cpl-ir/src/lower/call.rs` (13.4k lines) string-matches 89 names in match arms (`ir_builtin_names`, the builtin count agreed with tech-debt); 40 are public std names (`sort`, `sorted`, `first`, `sum`, `prefix`, `runs`, ...), so the std body may not be what runs (`measure.py privilege`). `crates/cpl-check/src/prelude.rs` tables also list some std names (`Counter`, `Subset`, ...).
- **Stubs.** 189 library-body and 105 `@test` `todo` (`measure.py todo-split`); largest in `tree`, `graph`, `geometry`, `strings` (`cpl-conform docs --causes`, R0402: 146 examples reach one).
- **Examples ahead of the compiler.** 174 examples pass an int range where a domain is named (E0204), 89 fill `[D]T` from a list literal (R0001), e.g. in `std/dsu.cp`, `std/beats.cp`. `flow` and `dsu` pass 0 examples.
- **Standalone check.** 28 of 40 files fail `cpl check std/<m>.cp` (418 errors), including generic code the checker can't yet type (`std/GAPS.md`, 88 entries).
- **Prelude size.** 192 names vs the 80 in `scripts/check.sh` (a warning, not a failure).
- **Doc gaps.** 32 public fns missing labels; `ternary_max`/`ternary_min` in `std/prelude.cp` are in no spec chapter (`spec-stub.py --check`, 13 problems).
- **Big files.** `std/geometry.cp` 3599, `std/seq.cp` 3373, `std/strings.cp` 3347 lines.
- **Location.** Std lives under `spec/` and is bundled into the compiler at build time; the move to `std/` and the removal of `CPL_SPEC`/`CPL_STD` mode switches are in `plan/ARCHITECTURE.md` (tech-debt runs it).
- **Untracked no-privilege test.** No test copies a std module into a user library and runs it.
