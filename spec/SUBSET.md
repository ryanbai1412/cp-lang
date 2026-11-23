# The Milestone 1 subset

What CP Lang promises for Milestone 1 (`plan/product/MILESTONE-1.md`): a
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
| R01.1 | 1.1 Source text | supported | 7 pass |
| R01.2 | 1.2 Layout | supported | 14 pass |
| R01.3 | 1.3 Names | supported | 16 pass |
| R01.4 | 1.4 Literals | supported | 10 pass |
| R01.5 | 1.5 Operators and precedence | supported | 16 pass |
| R01.6 | 1.6 Statements | supported | 19 pass, 2 known gaps |
| R01.7 | 1.7 Control flow | supported | 8 pass |
| R01.8 | 1.8 Expressions | supported | 10 pass |
| R01.9 | 1.9 Items | supported | 1 pass |
| R01.10 | 1.10 Grammar | supported | 10 pass |
| R01.11 | 1.11 Rejected spellings | supported | 4 pass, 1 known gaps |
| R02.1 | 2.1 The kinds of value | supported | 9 pass |
| R02.2 | 2.2 Scalars | supported | 4 pass |
| R02.3 | 2.3 Strings | supported | 4 pass, 2 known gaps |
| R02.4 | 2.4 Optionals | supported | 18 pass, 3 known gaps |
| R02.5 | 2.5 Tuples and records | supported | 11 pass |
| R02.6 | 2.6 Structs | supported | 16 pass, 2 known gaps |
| R02.7 | 2.7 Enums | supported | 15 pass, 1 known gaps |
| R02.8 | 2.8 Arrays and lists | supported | 12 pass |
| R02.9 | 2.9 Fresh types: domains and runtime-parameterized types | supported | 9 pass |
| R02.10 | 2.10 Type inference | supported | 6 pass |
| R02.11 | 2.11 Never implicit | supported | 10 pass |
| R03.1 | 1. Where widths come from | supported | 26 pass, 2 known gaps |
| R03.2 | 2. Spare values | supported | 13 pass |
| R03.3 | 3. Operations on niche types | supported | 6 pass |
| R03.4 | 4. Overflow, in one table | supported | 9 pass |
| R03.5 | 5. Modular numbers | supported | 6 pass |
| R04.2 | Domains | supported | 42 pass, 10 known gaps |
| R04.3 | Positions and cuts move the same way | supported | 7 pass |
| R04.4 | Two kinds of range, one type | supported | 38 pass, 5 known gaps |
| R04.7 | Middle of a range | supported | 2 pass |
| R04.8 | Decided (decision 137) | supported | 34 pass |
| R05.2 | Open-ended ranges | supported | 9 pass |
| R05.3 | 3. Bindings and places | supported | 17 pass |
| R05.6 | 6. `lends` fns and projections | supported | 8 pass, 1 known gaps |
| R05.7 | 7. Lent values (R6 to R8) | supported | 41 pass, 5 known gaps |
| R05.8 | 8. Overlap and order (R9 to R11) | supported | 20 pass, 2 known gaps |
| R05.9 | 9. Generators and views (R12) | supported | 15 pass, 4 known gaps |
| R05.10 | 10. Closures and nested fns | supported | 37 pass |
| R06.1 | 6.1 Functions | supported | 28 pass, 1 known gaps |
| R06.2 | 6.2 Nested functions and lambdas | supported | 14 pass, 1 known gaps |
| R06.3 | 6.3 Generators | supported | 12 pass |
| R06.4 | 6.4 Decorators | supported | 12 pass, 9 known gaps |
| R06.5 | 6.5 Where state lives | supported | 1 pass |
| R07.1 | Key decisions | supported | 13 pass, 1 known gaps |
| R07.2 | Terms | supported | 24 pass, 3 known gaps |
| R07.3 | The capabilities | supported | 8 pass, 1 known gaps |
| R07.4 | Slices: which keys survive | supported | 23 pass, 1 known gaps |
| R07.5 | Under the hood | supported | 17 pass, 2 known gaps |
| R07.6 | Generators | supported | 10 pass |
| R07.7 | Adapters (each makes a view: \(O(1)\), lazy) | supported | 19 pass |
| R07.8 | Making containers | supported | 1 pass |
| R07.9 | Folds | supported | 9 pass, 2 known gaps |
| R07.11 | Ordering and grouping | supported | 8 pass |
| R07.12 | More vocabulary | supported | 10 pass |
| R08.1 | Folds | supported | 30 pass, 3 known gaps |
| R08.2 | 8.2 Generics | supported | 19 pass, 2 known gaps |
| R08.3 | 8.3 Laws | supported | 5 pass |
| R08.4 | 8.4 Record row types (trial) | supported | 11 pass |
| R08.5 | 8.5 What is deliberately missing | supported | 10 pass |
| R09.1 | 9.1 Contexts | supported | 14 pass, 2 known gaps |
| R09.2 | 9.2 Build modes | supported | 5 pass, 3 known gaps |
| R09.3 | 9.3 What is checked | supported | 18 pass, 4 known gaps |
| R09.4 | 9.4 Turning checks on and off | supported | 9 pass |
| R09.5 | 9.5 Trust tiers | supported | 5 pass |
| R09.6 | 9.6 Recursion and the stack | supported | 1 pass |
| R10.1 | 10.1 Program shape | supported | 15 pass, 1 known gaps |
| R10.2 | 10.2 Reading | supported | 44 pass |
| R10.3 | 10.3 Printing | supported | 20 pass, 1 known gaps |
| R11.1 | 11.1 Program files and library files | supported | 7 pass, 3 known gaps |
| R11.2 | 11.2 `use` | supported | 8 pass, 1 known gaps |
| R11.3 | 11.3 The prelude | supported | 17 pass, 3 known gaps |
| R11.4 | 11.4 Versions, pins and migration | documented | `cpl migrate` and pinned rebuilds wait (milestone: "full backward compatibility" can wait) |
| R12.4 | W4. Sequences, views and index spaces (P4) | documented | user-library powers: specified, not promised for the milestone |
| R12.11 | W10a. Records, associated types and inline bounds (P10) | documented | user-library powers: specified, not promised for the milestone |
| R13.1 | 13.1 The output file | supported | 5 pass, 1 known gaps |
| R13.2 | 13.2 Values | supported | 14 pass |
| R13.3 | 13.3 Functions, closures, generators | supported | 7 pass |
| R04a.2 | A. Order-only uses: no key is ever observed | supported | 8 pass, 1 known gaps |
| R04a.3 | B. An index found inside the slice, used outside it | supported | 3 pass, 1 known gaps |
| R04a.4 | C. An outside index used on the slice | supported | 4 pass, 1 known gaps |
| R04a.5 | D. Counters and literals into the slice | supported | 9 pass |
| R04a.6 | E. Generic algorithms run on a sub-sequence | supported | 3 pass |
| R04a.7 | F. Grids and multi-dimensional arrays | supported | 3 pass, 1 known gaps |
| R04a.8 | G. Revision: changing your mind | supported | 3 pass, 1 known gaps |
| R04a.9 | H. Keys of the other views | supported | 9 pass, 3 known gaps |
| R04a.11 | Recommendation: identity everywhere (I) | supported | 12 pass, 3 known gaps |
| R04b.1 | Methods | supported | 38 pass, 8 known gaps |
| R04b.3 | User helpers | supported | 3 pass |
| R04b.4 | Library code | supported | 5 pass, 1 known gaps |
| R04b.5 | Spec changes this implies | supported | 17 pass, 3 known gaps |
| Rbundle | Bundling and submission | supported | 3 pass |
| Rcli | The `cpl` command | supported | 6 pass |
| Rfmt | `cpl fmt` | supported | 2 pass |
| Rjudge | Judge profiles | supported | 3 pass |
| Rlint | `cpl lint` | supported | 3 pass |
| Rlsp | Language server and editors | supported | 0 pass |
| std-algebra | library: algebra | documented | stdlib: outside the Milestone 1 contest staples |
| std-bag | library: bag | supported | 49 pass |
| std-beats | library: beats | documented | stdlib: outside the Milestone 1 contest staples |
| std-bitset | library: bitset | supported | 15 pass |
| std-compress | library: compress | documented | stdlib: wrappers returning `compress`'s fresh domain; `compress` itself is R07.11.5 (supported, std-seq) |
| std-deque | library: deque | supported | 22 pass |
| std-derive | library: derive | documented | stdlib: outside the Milestone 1 contest staples |
| std-dp | library: dp | supported | 21 pass, 1 known gaps |
| std-dsu | library: dsu | supported | 19 pass |
| std-fenwick | library: fenwick | supported | 13 pass |
| std-flow | library: flow | documented | stdlib: outside the Milestone 1 contest staples |
| std-geometry | library: geometry | documented | stdlib: outside the Milestone 1 contest staples |
| std-graph | library: graph | supported | 88 pass, 2 known gaps |
| std-grid | library: grid | supported | 10 pass, 1 known gaps |
| std-hashmap | library: hashmap | supported | 37 pass |
| std-heap | library: heap | supported | 60 pass |
| std-interval | library: interval | supported | 13 pass |
| std-io | library: io | supported | 4 pass |
| std-lazy | library: lazy | supported | 10 pass |
| std-marks | library: marks | supported | 18 pass |
| std-math | library: math | supported | 51 pass |
| std-matrix | library: matrix | documented | stdlib: outside the Milestone 1 contest staples |
| std-mo | library: mo | documented | stdlib: outside the Milestone 1 contest staples |
| std-mod | library: mod | supported | 25 pass |
| std-ntt | library: ntt | documented | stdlib: outside the Milestone 1 contest staples |
| std-opt | library: opt | documented | stdlib: outside the Milestone 1 contest staples |
| std-pair | library: pair | documented | stdlib: outside the Milestone 1 contest staples |
| std-persistent | library: persistent | documented | stdlib: outside the Milestone 1 contest staples |
| std-poly | library: poly | documented | stdlib: outside the Milestone 1 contest staples |
| std-prefix | library: prefix | supported | 7 pass, 1 known gaps |
| std-rand | library: rand | documented | stdlib: outside the Milestone 1 contest staples |
| std-search | library: search | supported | 18 pass |
| std-segtree | library: segtree | supported | 22 pass, 1 known gaps |
| std-seq | library: seq | supported | 108 pass |
| std-slope | library: slope | documented | stdlib: outside the Milestone 1 contest staples |
| std-sparse | library: sparse | supported | 3 pass |
| std-spec | library: spec | documented | stdlib: outside the Milestone 1 contest staples |
| std-strings | library: strings | supported | 68 pass, 1 known gaps |
| std-subset | library: subset | documented | stdlib: outside the Milestone 1 contest staples |
| std-timetree | library: timetree | documented | stdlib: outside the Milestone 1 contest staples |
| std-tree | library: tree | supported | 63 pass, 3 known gaps |
| std-window | library: window | supported | 10 pass |
| Rstress | Stress, shrinking and laws | supported | 2 pass |

## Known gaps

- core-01/compound-assign-key-evaluated-once.cp: compiler
- core-01/bool-caret-names-ne.cp: diagnostics
- core-01/continuation-leading-or.cp: compiler
- core-01/list-repeat-times-n.cp: compiler
- core-01/lone-cr-in-string-is-error.cp: compiler
- core-01/scientific-fraction-is-int.cp: compiler
- core-01/stepped-range-is-a-value.cp: compiler
- core-02/enum-method-is-variant.cp: compiler
- core-02/getter-needs-defaults.cp: compiler
- core-02/question-extent-let-stays-optional.cp: compiler
- core-02/question-not-early-return.cp: compiler
- core-02/spec-block-enums.cp: compiler
- core-02/str-len-runtime-bound-rejects.cp: compiler
- core-02/str-len-runtime-bound.cp: compiler
- core-02/str-len-runtime-bound.cp: compiler
- core-02/char-of-non-ascii-traps.cp: compiler
- core-02/unwrap-shadow-below-multitest.cp: compiler
- core-02/bare-variant-needs-expected-enum.cp: compiler
- core-02/counted-array-never-grows.cp: compiler
- core-02/read-str-of-alphabet-type-arg.cp: compiler
- core-03/annotation-widens-fresh-read.cp: compiler
- core-03/field-bound-from-input.cp: compiler
- core-03/memory-over-profile.cp: compiler
- core-04/a11-rebase-reads-from-zero.cp: compiler
- core-04/a7-consume-front.cp: compiler
- core-04/b4-prefix-of-range.cp: compiler
- core-04/cuts-domain-has-range.cp: compiler
- core-04/f2-sub-rectangle-offset.cp: compiler
- core-04/grid-keys-are-cells.cp: compiler
- core-04/h-step-keeps-source-keys.cp: compiler
- core-04/h-step-skipped-key-traps.cp: compiler
- core-04/inline-product-domain-is-seq.cp: compiler
- core-04/keyed-counter-is-not-a-key.cp: compiler
- core-04/list-of-domain-array-loses-domain.cp: compiler
- core-04/nested-slice-below-view-start-traps.cp: compiler
- core-04/pair-symmetric.cp: compiler
- core-04/palindromes-longest-at-key.cp: stdlib
- core-04/piece-into-fold.cp: compiler
- core-04/prefix-and-z-function-over-domain-array-and-range.cp: compiler
- core-04/prod-at-neighbour.cp: compiler
- core-04/range-key-is-element.cp: compiler
- core-04/range-key-outside-traps.cp: compiler
- core-04/range-len-walk-ends.cp: compiler
- core-04/rebase-fresh-domain.cp: compiler
- core-04/rebased-str-indexes-from-zero.cp: stdlib
- core-04/runtime-domain-literal-between-traps.cp: compiler
- core-04/spec-block-drop-rest.cp: compiler
- core-04/spec-block-first-drop-next-greater.cp: compiler
- core-04/spec-block-split-by-search.cp: compiler
- core-04/zip-keyed-by-first.cp: compiler
- core-04/char-domain-position-to-char.cp: compiler
- core-04/table-one-cell-parameter.cp: compiler
- core-04/range-type-binds-generic-domain.cp: compiler
- core-05/at-assign-needs-var.cp: compiler
- core-05/closure-capture-blocks-push.cp: compiler
- core-05/exercise-04-var-from-lent-param.cp: compiler
- core-05/exercise-13-disjoint-slices-meet.cp: compiler
- core-05/exercise-20-segtree-lent-from-ws.cp: compiler
- core-05/exercise-24-mut-generator-rewalked-lint.cp: compiler
- core-05/exercise-33-lends-result-loop-writes-source.cp: compiler
- core-05/exercise-35-lent-optional-kept.cp: compiler
- core-05/lent-generator-blocks-overlapping-write.cp: compiler
- core-05/slices-proved-apart.cp: compiler
- core-05/spec-block-segtree-of-ws.cp: compiler
- core-05/file-body-var-holds-lent.cp: compiler
- core-05/lent-var-reassigned-from-itself.cp: compiler
- core-06/alt-main-is-removed.cp: compiler
- core-06/brute-main-is-removed.cp: compiler
- core-06/checker-is-removed.cp: compiler
- core-06/gen-array-over-domain.cp: compiler
- core-06/gen-graph-simple-connected.cp: compiler
- core-06/gen-int-in-range.cp: compiler
- core-06/gen-perm.cp: compiler
- core-06/gen-rooted-shape.cp: compiler
- core-06/gen-str-of-alphabet.cp: compiler
- core-06/check-read-output-type-argument.cp: compiler
- core-07/fold-with-init.cp: compiler
- core-07/assign-into-range.cp: compiler
- core-07/sorted-has-no-desc-flag.cp: stdlib
- core-07/fold-without-parens-warns.cp: compiler
- core-07/grid-row-index-and-block-order.cp: compiler
- core-07/prefix-scan-cuts.cp: compiler
- core-07/rev-keeps-keys.cp: stdlib
- core-07/range-folds-are-ranges-own.cp: stdlib
- core-07/slice-past-source-traps.cp: compiler
- core-07/tally-is-counter.cp: stdlib
- core-07/tally-missing-key-is-zero.cp: stdlib
- core-07/heap-by-desc-lambda.cp: compiler
- core-08/const-param-arithmetic.cp: compiler
- core-08/extension-concrete-and-generic-ambiguous.cp: compiler
- core-08/static-fn-default-param-mismatch.cp: compiler
- core-08/trait-message-bound-holds.cp: compiler
- core-09/release-division-by-zero-fallback.cp: compiler
- core-09/release-list-index-unchecked.cp: perf
- core-09/release-skips-assert.cp: compiler
- core-09/stress-finds-broken-law.cp: compiler
- core-09/user-context-lexical-without-uses.cp: compiler
- core-09/uses-not-inferred-top-level.cp: compiler
- core-10/print-list-of-pairs-one-line.cp: compiler
- core-09/deep-recursion-trap.cp: compiler
- core-10/show-impl-enum.cp: compiler
- core-10/test-position-stored-above-multitest.cp: compiler
- core-10/read-range-arithmetic-lower-bound.cp: compiler
- core-10/print-view-honours-sep.cp: compiler
- std-graph/bfs-over-named-product-domain.cp: compiler
- core-05/let-heap-element-lends-no-copy.cp: compiler
- core-06/generator-in-nested-for-streams.cp: compiler
- core-04/product-result-indexed-by-parts.cp: compiler
- core-06/lambda-unpacks-tuple-vertex.cp: compiler
- core-10/print-range-prints-items.cp: compiler
- core-02/str-len-and-charset-compose.cp: compiler
- core-02/list-of-str-is-chars.cp: compiler
- std-graph/undirected-unweighted-has-deg.cp: stdlib
- core-10/tuple-dependent-bound-in-block.cp: compiler
- std-seq/min-of-empty-list-is-none.cp: compiler
- core-11/mod-uses-sibling.cp: compiler
- core-11/prelude-has-components.cp: compiler
- core-11/runtime-domain-use-before-definition.cp: compiler
- core-11/strings-table-in-prelude.cp: compiler
- core-13/build-warns-near-size-limit.cp: compiler
- diagnostics/graph-edge-field-names.cp: diagnostics
- diagnostics/sum-of-str.cp: diagnostics
- std-graph/dfs-shares-seen.cp: stdlib
- std-graph/user-impl-next-weighted-arc-runs-dijkstra.cp: stdlib
- std-dp/interval-ranges-deque-game.cp: stdlib
- std-prefix/first-ge-negative-element-traps.cp: stdlib
- std-segtree/closure-op-picks-index.cp: compiler
- std-strings/split-trailing-and-leading.cp: stdlib
- std-seq/split-on-separator-predicate.cp: stdlib
- core-07/range-in-brackets-help-names-list.cp: diagnostics
- std-rand/shuffled-help-names-rng-shuffle.cp: diagnostics
- core-06/check-multitest-reads-own-shape.cp: tools
- std-graph/min-mean-cycle-acyclic-is-none.cp: stdlib
- std-heap/drain-break-keeps-the-rest.cp: compiler
