# The Milestone 1 subset

What CP Lang promises for Milestone 1 (`handbook/product/milestones.md`): a
contestant who has never used it learns, solves, tests and submits with it,
unaided. Each rule section of `core/` and `tools/` is one row:

- *supported*: promised. Every conformance case citing it passes, except the
  cases listed under Known gaps, each with the domain that owns the fix.
- *documented*: specified, not promised for the milestone; it may work.
- *gap*: not usable yet as a whole; the note names the owner.

`scripts/conformance-subset.py --check` (it runs `cpl-conform`;
`--board <scoreboard>` reuses a run) holds this file to the suite: a failing case in a supported section must be a known gap, and
a known gap that starts passing must be dropped. `--draft` regenerates the
table and the list; statuses and owner notes are kept.

Library families are rows too, one per `conformance/std-*` folder
(`std-heap` is every case there). The contest staples (heaps, graphs and
trees, DSU, prefix sums, Fenwick and segment trees, sequences, strings,
grids, modular math, I/O, search and DP) are supported; the rest are
documented. The stdlib's own gaps are in `std/GAPS.md`.

`Rlsp` cases drive `cpl lsp --stdio` (`tools-lsp/`); the LSP's own suites go deeper (`tools/lsp.md`).

## Sections

| Section | Heading | Status | Note |
|---|---|---|---|
| R01.1 | 1.1 Source text | supported | 12 pass |
| R01.2 | 1.2 Layout | supported | 18 pass |
| R01.3 | 1.3 Names | supported | 26 pass |
| R01.4 | 1.4 Literals | supported | 12 pass |
| R01.5 | 1.5 Operators and precedence | supported | 24 pass |
| R01.6 | 1.6 Statements | supported | 30 pass |
| R01.7 | 1.7 Control flow | supported | 21 pass |
| R01.8 | 1.8 Expressions | supported | 19 pass, 2 known gaps |
| R01.9 | 1.9 Items | supported | 2 pass |
| R01.10 | 1.10 Grammar | supported | 17 pass |
| R01.11 | 1.11 Rejected spellings | supported | 5 pass |
| R02.1 | 2.1 The kinds of value | supported | 11 pass |
| R02.2 | 2.2 Scalars | supported | 9 pass |
| R02.3 | 2.3 Strings | supported | 10 pass |
| R02.4 | 2.4 Optionals | supported | 26 pass, 1 known gaps |
| R02.5 | 2.5 Tuples and records | supported | 11 pass |
| R02.6 | 2.6 Structs | supported | 19 pass |
| R02.7 | 1.3 Names | supported | 21 pass |
| R02.8 | 2.8 Arrays and lists | supported | 17 pass |
| R02.9 | 2.9 Fresh types: domains and runtime-parameterized types | supported | 12 pass |
| R02.10 | 2.10 Type inference | supported | 8 pass |
| R02.11 | 2.11 Never implicit | supported | 10 pass |
| R03.1 | 1. Where widths come from | supported | 34 pass, 1 known gaps |
| R03.2 | 2. Spare values | supported | 13 pass |
| R03.3 | 3. Operations on niche types | supported | 6 pass |
| R03.4 | 4. Overflow, in one table | supported | 56 pass |
| R03.5 | 5. Modular numbers | supported | 7 pass |
| R04.2 | The model in one paragraph | supported | 100 pass, 4 known gaps |
| R04.3 | Positions and cuts move the same way | supported | 11 pass |
| R04.4 | Two kinds of range, one type | supported | 56 pass, 2 known gaps |
| R04.7 | Middle of a range | supported | 2 pass |
| R04.8 | Decided (decision 137) | supported | 41 pass |
| R05.2 | Open-ended ranges | supported | 9 pass |
| R05.3 | 3. Bindings and places | supported | 24 pass, 2 known gaps |
| R05.4 | 4. Parameters and calls (R4) | supported | 9 pass |
| R05.5 | 5. Returns (R5) | supported | 4 pass |
| R05.6 | 6. `lends` fns and projections | supported | 11 pass |
| R05.7 | 7. Lent values (R6 to R8) | supported | 87 pass, 5 known gaps |
| R05.8 | Contest code | supported | 32 pass |
| R05.9 | 9. Generators and views (R12) | supported | 22 pass, 2 known gaps |
| R05.10 | 10. Closures and nested fns | supported | 39 pass |
| R06.1 | 6.1 Functions | supported | 43 pass, 1 known gaps |
| R06.2 | 6.2 Nested functions and lambdas | supported | 17 pass |
| R06.3 | 6.3 Generators | supported | 14 pass |
| R06.4 | 6.4 Decorators | supported | 35 pass, 1 known gaps |
| R06.5 | 6.5 Where state lives | supported | 5 pass |
| R07.1 | Key decisions | supported | 23 pass |
| R07.2 | Terms | supported | 29 pass |
| R07.3 | User helpers | supported | 19 pass, 1 known gaps |
| R07.4 | Slices: which keys survive | supported | 51 pass, 1 known gaps |
| R07.5 | The rules | supported | 33 pass |
| R07.6 | Generators | supported | 12 pass |
| R07.7 | Adapters (each makes a view: \(O(1)\), lazy) | supported | 27 pass, 1 known gaps |
| R07.8 | Making containers | supported | 4 pass |
| R07.9 | Folds | supported | 25 pass, 3 known gaps |
| R07.11 | Ordering and grouping | supported | 15 pass, 1 known gaps |
| R07.12 | More vocabulary | supported | 12 pass |
| R08.1 | Folds | supported | 39 pass, 2 known gaps |
| R08.2 | 8.2 Generics | supported | 28 pass |
| R08.3 | 8.3 Laws | supported | 5 pass |
| R08.4 | 8.4 Record row types (trial) | supported | 11 pass |
| R08.5 | 1.4 Literals | supported | 11 pass |
| R09.1 | 9.1 Contexts | supported | 17 pass |
| R09.2 | 9.2 Build modes | supported | 7 pass, 1 known gaps |
| R09.3 | 9.3 What is checked | supported | 26 pass, 1 known gaps |
| R09.4 | Domains | supported | 13 pass, 1 known gaps |
| R09.5 | 9.5 Trust tiers | supported | 5 pass |
| R09.6 | 9.6 Recursion and the stack | supported | 1 pass |
| R10.1 | 6.1 Functions | supported | 21 pass, 1 known gaps |
| R10.2 | 10.2 Reading | supported | 59 pass |
| R10.3 | 10.3 Printing | supported | 34 pass |
| R11.1 | 11.1 Program files and library files | supported | 15 pass |
| R11.2 | 11.2 `use` | supported | 13 pass |
| R11.3 | 11.3 The prelude | supported | 24 pass, 2 known gaps |
| R11.4 | 11.4 Versions, pins and migration | documented | `cpl migrate` and pinned rebuilds wait (milestone: "full backward compatibility" can wait) |
| R12.4 | W4. Sequences, views and index spaces (P4) | documented | user-library powers: specified, not promised for the milestone |
| R12.11 | W10a. Records, associated types and inline bounds (P10) | documented | user-library powers: specified, not promised for the milestone |
| R13.1 | 13.1 The output file | supported | 6 pass |
| R13.2 | 13.2 Values | supported | 14 pass |
| R13.3 | 13.3 Functions, closures, generators | supported | 9 pass, 1 known gaps |
| R04a.2 | A. Order-only uses: no key is ever observed | supported | 9 pass |
| R04a.3 | B. An index found inside the slice, used outside it | supported | 4 pass |
| R04a.4 | C. An outside index used on the slice | supported | 5 pass |
| R04a.5 | D. Counters and literals into the slice | supported | 9 pass |
| R04a.6 | E. Generic algorithms run on a sub-sequence | supported | 3 pass |
| R04a.7 | F. Grids and multi-dimensional arrays | supported | 4 pass |
| R04a.8 | G. Revision: changing your mind | supported | 3 pass, 1 known gaps |
| R04a.9 | H. Keys of the other views | supported | 13 pass |
| R04a.11 | Recommendation: identity everywhere (I) | supported | 14 pass, 1 known gaps |
| R04b.1 | Methods | supported | 49 pass, 3 known gaps |
| R04b.3 | User helpers | supported | 3 pass |
| R04b.4 | Library code | supported | 6 pass |
| R04b.5 | Spec changes this implies | supported | 19 pass, 1 known gaps |
| Rbundle | Bundling and submission | supported | 4 pass, 3 known gaps |
| Rcli | The `cpl` command | supported | 6 pass |
| Rfmt | `cpl fmt` | supported | 2 pass |
| Rjudge | Judge profiles | supported | 11 pass |
| Rlint | `cpl lint` | supported | 3 pass |
| Rlsp | Language server and editors | supported | 9 pass |
| std-algebra | library: algebra | documented | stdlib: outside the Milestone 1 contest staples |
| std-bag | library: bag | supported | 54 pass, 1 known gaps |
| std-beats | library: beats | documented | stdlib: outside the Milestone 1 contest staples |
| std-bitset | library: bitset | supported | 16 pass |
| std-compress | library: compress | documented | stdlib: wrappers returning `compress`'s fresh domain; `compress` itself is R07.11.5 (supported, std-seq) |
| std-deque | library: deque | supported | 22 pass |
| std-derive | library: derive | documented | stdlib: outside the Milestone 1 contest staples |
| std-dp | library: dp | supported | 24 pass |
| std-dsu | library: dsu | supported | 21 pass |
| std-fenwick | library: fenwick | supported | 14 pass |
| std-flow | library: flow | documented | stdlib: outside the Milestone 1 contest staples |
| std-geometry | library: geometry | documented | stdlib: outside the Milestone 1 contest staples |
| std-graph | library: graph | supported | 99 pass, 1 known gaps |
| std-grid | library: grid | supported | 11 pass |
| std-hashmap | library: hashmap | supported | 38 pass |
| std-heap | library: heap | supported | 67 pass |
| std-interval | library: interval | supported | 13 pass |
| std-io | library: io | supported | 4 pass |
| std-lazy | library: lazy | supported | 11 pass |
| std-marks | library: marks | supported | 18 pass |
| std-math | library: math | supported | 53 pass |
| std-matrix | library: matrix | documented | stdlib: outside the Milestone 1 contest staples |
| std-mo | library: mo | documented | stdlib: outside the Milestone 1 contest staples |
| std-mod | library: mod | supported | 27 pass |
| std-ntt | library: ntt | documented | stdlib: outside the Milestone 1 contest staples |
| std-opt | library: opt | documented | stdlib: outside the Milestone 1 contest staples |
| std-pair | library: pair | documented | stdlib: outside the Milestone 1 contest staples |
| std-persistent | library: persistent | documented | stdlib: outside the Milestone 1 contest staples |
| std-poly | library: poly | documented | stdlib: outside the Milestone 1 contest staples |
| std-prefix | library: prefix | supported | 21 pass |
| std-rand | library: rand | documented | stdlib: outside the Milestone 1 contest staples |
| std-search | library: search | supported | 20 pass, 1 known gaps |
| std-segtree | library: segtree | supported | 27 pass, 1 known gaps |
| std-seq | library: seq | supported | 189 pass, 1 known gaps |
| std-slope | library: slope | documented | stdlib: outside the Milestone 1 contest staples |
| std-sparse | library: sparse | supported | 3 pass |
| std-spec | library: spec | documented | stdlib: outside the Milestone 1 contest staples |
| std-strings | library: strings | supported | 71 pass |
| std-subset | library: subset | documented | stdlib: outside the Milestone 1 contest staples |
| std-timetree | library: timetree | documented | stdlib: outside the Milestone 1 contest staples |
| std-tree | library: tree | supported | 71 pass, 2 known gaps |
| std-window | library: window | supported | 12 pass, 1 known gaps |
| Rstress | Stress, shrinking and laws | supported | 2 pass |
| Rtrace | Traces, the debugger and runtime errors | supported | 5 pass |

## Known gaps

- core-01/comprehension-filter-if-let.cp: compiler
- core-01/stepped-range-is-a-value.cp: compiler
- core-03/memory-over-profile.cp: compiler
- core-04/grid-keys-are-cells.cp: compiler
- core-04/grid-rows-is-a-getter.cp: compiler (E0304 for a std `@get` fn called with `()`)
- core-04/keyed-counter-is-not-a-key.cp: compiler
- core-04/library-domain-count-trap-names-sizing-site-release.cp: compiler
- core-04/list-fill-copies-per-key.cp: compiler
- core-04/list-of-domain-array-loses-domain.cp: compiler
- core-04/palindromes-longest-at-key.cp: stdlib
- core-04/pieces-of-a-domain-take-its-cuts.cp: compiler
- core-04/rebased-str-indexes-from-zero.cp: stdlib
- core-04/runtime-domain-literal-between-traps.cp: compiler
- core-04/table-one-cell-parameter.cp: compiler
- core-04/zip-keyed-by-first.cp: compiler
- core-05/exercise-04-var-from-lent-param.cp: compiler
- core-05/exercise-20-segtree-lent-from-ws.cp: compiler
- core-05/exercise-35-lent-optional-kept.cp: compiler
- core-05/lent-generator-blocks-overlapping-write.cp: compiler
- core-05/let-heap-element-lends-no-copy.cp: compiler
- core-05/map-entry-index-key-lends.cp: compiler
- core-05/spec-block-segtree-of-ws.cp: compiler
- core-06/gen-graph-simple-connected.cp: stdlib (generated edge record has no `all`)
- core-07/chain-takes-any-number.cp: stdlib
- core-07/count-of-bools-free-form.cp: compiler
- core-07/count-of-bools-needs-a-predicate.cp: compiler
- core-07/fold-with-init.cp: compiler
- core-07/sorted-has-no-desc-flag.cp: compiler (the IR lowers `desc:` itself)
- core-08/extension-concrete-and-generic-ambiguous.cp: compiler
- core-08/trait-message-bound-holds.cp: compiler
- core-09/release-list-index-unchecked.cp: perf
- core-10/test-position-stored-above-multitest.cp: compiler
- core-11/prelude-has-components.cp: compiler
- core-11/strings-table-in-prelude.cp: compiler
- diagnostics/graph-edge-field-names.cp: diagnostics
- std-algebra/each-of-bool-operators.cp: stdlib
- std-bag/range-view-keeps-the-bag-lent.cp: compiler lends of a std method returning a struct
- std-graph/user-impl-next-weighted-arc-runs-dijkstra.cp: stdlib
- std-search/real-bsearch-is-a-range-method.cp: stdlib `Range<F64>.bsearch/peak/valley(steps:)` replace `bsearch_real`/`ternary_*_real` (decision 419)
- std-segtree/closure-op-picks-index.cp: stdlib `all` of an identity-less op is `?T`; needs identity-less fold typing (plan/QUESTIONS.md)
- std-seq/split-on-separator-predicate.cp: stdlib
- std-window/track-infers-element-from-sequence.cp: compiler
- tools-bundle/vendor-copies-pinned-libraries.cp: tools Milestone 2 (pins, Rbundle.6), not promised for Milestone 1
