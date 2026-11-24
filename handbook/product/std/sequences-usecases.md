# Sequences: use cases

| | |
|---|---|
| Kind | Use cases |
| Owner | stdlib |
| Surface | [sequences.md](sequences.md) |

Each entry is one need, grouped by what you are doing. The ID names the need,
so a design change keeps it. A supported or partial entry names its program:
`spec/conformance/std-seq/seq-<id>-*.cp`, which runs in CI. Where a use case
states a cost, a separate `-cost` program runs it at \(n = 10^6\), where the
wrong complexity times out. Repros of gaps were run against main on 2026-11-23.
[sequences-inventory.md](sequences-inventory.md) maps C++, Python, Rust and
the AtCoder Library onto these IDs.

## Search

Find a value, a position, or the cut where a property starts to hold. Values come from `a`, positions from `a.keys`; that one rule covers find, filter, max and first_true.

### seq/U1: value of the first or last match
Need: you need the element itself, e.g. the first value over a threshold.
Design: `a.find(p) -> ?T`, `a.rfind(p) -> ?T`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u01-first-match-value.cp`.

### seq/U2: position of the first or last match
Need: you need where a property starts to hold, in the array's own index space.
Design: `a.keys.find(i => p(a[i])) -> ?I`, `a.keys.rfind(..)`: positions are a search over `keys`, one spelling for every position question.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u02-first-match-position.cp`.

### seq/U26: every position that matches
Need: list the indices of all zeros.
Design: `a.keys.filter(i => p(a[i]))`, lazy; `collect()` builds the list.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u26-all-match-positions.cp`.

### seq/U70: a position can't index the wrong array
Need: two arrays of the same length over different index spaces; using one's position on the other is a bug.
Design: a position from `a.keys` has `a`'s domain type; indexing an array over another domain is E0301.
Milestone: M1-critical. Status: supported
Evidence: reject case `spec/conformance/std-seq/seq-u70-position-wrong-domain-rejected.cp`.

### seq/U5: transition point of a monotone predicate
Need: binary search on the answer.
Design: `(lo..hi).first_true(p) -> ?T`, `.last_true(p)`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u05-monotone-transition.cp`.

### seq/U6: bounds in sorted data
Need: `lower_bound`/`upper_bound` on a sorted array.
Design: `a.first_ge(x)`, `first_gt`, `last_le`, `last_lt` give `?T`; their positions are `a.keys.first_true(i => a[i] >= x)`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u06-sorted-bounds.cp`.

### seq/U27: binary search answered as a position or a cut
Need: the insertion point of `x`, or where the prefix stops being small.
Design: `a.keys.first_true(i => a[i] >= x) -> ?I`; `partition_point(p)` gives the count left of the cut.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u27-bsearch-as-cut.cp`.

### seq/U28: how many sorted values fall in `[lo, hi)`
Need: count values in a range after sorting once.
Design: `a.equal_range(x)` gives the positions equal to `x`; a general `[lo, hi)` count is `a.count_in(lo..hi)`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u28-count-in-value-range.cp`.

### seq/U29: binary search over reals
Need: the smallest real `x` with `f(x) >= t`.
Design: `(lo..hi).bsearch(p, steps: 60)` on `Range<F64>` (decision 419).
Milestone: M1-critical. Status: gap
Evidence: `(0.0..10.0).bsearch(..)` is E0302: a range's ends can't be `F64`.

### seq/U30: is it sorted, a permutation, a subsequence
Need: check a precondition or the answer itself.
Design: `a.is_sorted()`, `a.is_permutation(b)` (same multiset; `p.is_permutation(p.keys)` for a permutation of its keys), `a.is_subsequence(b)`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u30-is-sorted-perm-subseq.cp`.

### seq/U31: compare two sequences
Need: lexicographic comparison of answers, or the first position where two differ.
Design: `a < b`, `a == b`; the first difference is `a.keys.find(i => a[i] != b[i])`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u31-compare-sequences.cp`.

## Extremes and order statistics

The biggest, the smallest, the k-th, with a stated rule for ties and for empty input.

### seq/U4: extreme value, possibly empty
Need: the max of a list that may be empty, or by a key.
Design: `a.max(by:) -> ?T`, `a.min(by:) -> ?T`; `?? x` names the fallback; `max(f(i) for i in r)` over a generator.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u04-extreme-value.cp`.

### seq/U3: position of the extreme, with a tie rule
Need: argmax of a score, where the problem says "the leftmost" or "the rightmost".
Design: `a.argmax(by:, ties: Ties.Last) -> ?I`; `max` and `min` take the same `ties:`. Whether argmax is spelled `a.keys.max(by: i => a[i])` is the index-model proposal.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u03-extreme-position.cp`; widest ties are seq/U69.

### seq/U32: min and max in one pass
Need: the spread `max - min` of a list.
Design: `a.minmax() -> ?(T, T)`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u32-minmax-one-pass.cp`.

### seq/U33: how many times the extreme occurs
Need: count the positions holding the maximum.
Design: `a.max_count() -> (T, Int)`, `a.min_count()`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u33-count-of-extreme.cp`.

### seq/U34: k-th smallest and the k smallest
Need: median, or the three best scores.
Design: `a.select(k)` (in place, average \(O(n)\)); `a.min_k(k)`, `a.max_k(k)` as sorted lists.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u34-kth-and-top-k.cp`; `select` on a literal is E0401 at the call, asking for a `var`.

### seq/U35: majority element
Need: the value that fills more than half.
Design: `a.majority() -> ?T` (Boyer-Moore).
Milestone: later. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u35-majority.cp`.

### seq/U69: the maximum-sum segment and where it is
Need: Codeforces 2066F: the widest maximum-sum subarray, as a range.
Design: a `MaxSub` fold that returns its range; belongs to range structures, listed here because contestants look for it here.
Milestone: M1-differentiating. Status: gap
Evidence: the 2066F port hand-writes `widest_max`; `MaxSub` folds the sum only.

## Folds and scans

Fold the whole sequence once, or every prefix into a table read at cuts.

### seq/U18: count, sum, any, all, contains on any shape
Need: the one-line checks.
Design: `a.count(p)`, `.sum()`, `.any(p)`, `.all(p)`, `.contains(x)`, `.prod()`, `.mean()`; empty gives `0`, `false`, `true`, `1`, `None`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u18-aggregates-every-shape.cp`.

### seq/U9: a fold with a seed and a custom step
Need: count something with state as you walk.
Design: `a.fold((acc, x) => .., init: v)`: the seed is a named option.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u9-fold-seed.cp`.

### seq/U36: a fold with no identity
Need: fold with an op that has no neutral value, on a non-empty list.
Design: `a.reduce(f) -> ?T`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u36-fold-without-identity.cp`.

### seq/U63: two folds over one generator
Need: sum and max of `f(i)` without building a list.
Design: a generator is a value you can walk twice: `let g = r.map(f); g.sum(); g.max()`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u63-two-folds-one-pass.cp`.

### seq/U7: range folds from one prefix table
Need: sums of `a[l..r]` in \(O(1)\) after \(O(n)\).
Design: `a.scan(+)` then `.fold(l..r)`, `.prefix(c)`. A table from another op answers prefix reads, and a range read is an error; today it traps, the goal is a compile error.
Milestone: M1-critical. Status: partial (a trap until the type error exists)
Evidence: program `spec/conformance/std-seq/seq-u07-range-fold-table.cp`; trap case `spec/conformance/std-prefix/scan-min-range-fold-traps.cp`; no reject case yet.

### seq/U8: prefix folds over a slice
Need: the same table over part of the array.
Design: `a[l..r].scan(+)`, keyed by the slice's cuts.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u08-scan-over-slice.cp`.

### seq/U23: running extremes
Need: the max so far at each position.
Design: `a.scan(max)` read with `.prefix(c)` and `.all`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u23-running-extreme.cp`.

### seq/U37: suffix folds
Need: the best value to the right of each position.
Design: `a.rev.scan(f)`, on the same cuts; `suffix_fold(op, a)`.
Milestone: M1-critical. Status: partial
Evidence: program `spec/conformance/std-seq/seq-u37-suffix-folds.cp`; `suffix_fold(+, a)` is E0201: not in the prelude.

### seq/U38: fold of everything but one position
Need: product of all others, or best excluding `i`.
Design: `a.prefix_excl(op)`, or `pre[i.l]` combined with `suf[i.r]`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u38-prefix-excl-method.cp`; it still needs `use std.prefix.{prefix_excl}`.

### seq/U39: a scan of a scan
Need: running max of two prefix sums.
Design: `a.scan(+)` is indexable by cuts, so `[D.cuts] c => max(p[c], q[c])` then scan again.
Milestone: M1-differentiating. Status: partial
Evidence: program `spec/conformance/std-seq/seq-u39-combined-scan.cp`; the table of cuts is built by hand: `p.cuts` is E0203.

## Runs, groups and windows

Walk the sequence in pieces: maximal runs, groups by key, fixed windows and adjacent pairs. Pieces are views.

### seq/U10: runs of equal values or keys
Need: compress `aabccc` into blocks, with their lengths.
Design: `a.runs(by:)` yields each run as a view over `a`; also on `Str`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u10-runs.cp`, whose cuts are `r.keys.l` and `r.keys.r` as for any slice; cost program `seq-u10-runs-cost.cp`.

### seq/U40: runs walked backward with their cuts
Need: process blocks from the right, knowing where each starts.
Design: `a.rev.runs()`, each run carrying its cuts in `a`.
Milestone: M1-critical. Status: gap
Evidence: `[1, 1, 2].rev.runs()` is E0606: `Rev` is not `Indexed`.

### seq/U11: group by a key, in a defined order
Need: bucket values by `x % k`.
Design: `a.group(by:)` in ascending key order: \(O(n \log n)\), or \(O(n)\) buckets when the key is a domain.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u11-group-by-key.cp`.

### seq/U41: count each distinct value
Need: how many times each value appears.
Design: `a.tally()`, a map from value to count; over a domain, `[D]Int` counts.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u41-frequency-counts.cp`; the corpus hand-writes `cnt[x] += 1` in 60 ports.

### seq/U14: fixed windows, chunks and adjacent pairs
Need: sum of each length-`k` window, each block of `k`, or each difference.
Design: `a.windows(k)`, `a.chunks(k)`, `a.adjacent`, `a.cyclic_pairs()`, all views.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u14-windows-and-pairs.cp`; cost program `spec/conformance/std-seq/seq-u14-windows-and-pairs-cost.cp`; cost program `seq-u14-windows-cost.cp`.

### seq/U42: windows and pairs over a domain array
Need: the same over `[I]T`, keeping `I`.
Design: `windows`, `adjacent` and `zip(I, I.rev)` on domain arrays and domains.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u42-pieces-over-domain.cp`.

### seq/U43: split on a separator
Need: split a list on zeros into segments.
Design: `a.split(p)` yields slices.
Milestone: M1-critical. Status: supported
Evidence: `spec/conformance/std-seq/split-on-separator-predicate.cp` splits a list and a `Str` on a predicate.

### seq/U44: prefixes and suffixes by count or predicate
Need: the leading run of small values, or all but the first k.
Design: `take(k)`, `drop(k)`, `take_last(k)`, `drop_last(k)`, `take_while(p)`, `skip_while(p)`; views.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u44-take-drop-while.cp`.

### seq/U45: every k-th element
Need: odd positions, or a stride.
Design: `a.step(k)`.
Milestone: M1-critical. Status: partial
Evidence: program `spec/conformance/std-seq/seq-u45-step.cp`; on a view, `a.drop(1).step(2)` still resolves to the grid `step` (E0302).

## Sorting and permutations

Sort in place or into a copy, by key, stable; get the order, its inverse and ranks.

### seq/U12: sort or sorted by key, descending, stable
Need: order events by time, ties by input order.
Design: `a.sort(by:)`, `a.sorted(by:)`, stable; descending is a key, `by: x => desc(x)`; tuple keys for several.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u12-sort-by-key.cp`; cost program `spec/conformance/std-seq/seq-u12-sort-by-key-cost.cp`; cost program `seq-u12-sort-cost.cp`.

### seq/U46: sort by a user order
Need: structs with `impl Ord`.
Design: `sort()`, `sorted()`, `max()` follow the type's `Ord`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u46-user-ord.cp`.

### seq/U47: sort part of an array
Need: sort `a[l..r]` in place.
Design: `a[l..r].sort()`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u47-sort-slice.cp`.

### seq/U13: the order of a sort, and its inverse
Need: the rank of each element, or which index is the i-th smallest.
Design: `a.keys.sorted(by: i => a[i])` is the order; `.inverse()` gives ranks.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u13-sort-order-and-ranks.cp`.

### seq/U48: inverse of a product-keyed permutation
Need: a permutation of grid cells.
Design: `p.inverse()` on `[I, J](I, J)`.
Milestone: M1-differentiating. Status: gap
Evidence: `inverse()` on `[I, I]` is E0302.

### seq/U49: permutations: next, cycles, rank, parity
Need: walk all permutations, or decompose one into cycles.
Design: `a.next_permutation()`, `a.permutations()`, `p.perm_cycles()`, `p.perm_sign()`, `p.perm_rank()`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u49-permutation-tools.cp`.

### seq/U50: combinations, subsequences and products
Need: try every pair, every k-subset, every pair of options.
Design: `a.combinations(k)`, `a.subsequences()`, `a.pairs()`, `product(a, b)`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u50-combinations-subsets.cp`; `product` joined the prelude with R07.7.12.

### seq/U51: linear-time sorts
Need: sort a million small keys.
Design: `a.radix_sort(by:)`, `a.counting_sort(by:)`.
Milestone: later. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u51-linear-sorts.cp`; on a literal both are E0401 at the call, asking for a `var`.

### seq/U52: coordinate compression
Need: map values to ranks `0..k`.
Design: `compress(xs)` gives a domain and the map.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u52-compress-method.cp`; the free form is program `spec/conformance/std-seq/compress-and-pieces.cp`.

## Sets of sorted values

Sorted lists treated as sets: dedup, merge, union, intersection.

### seq/U19: distinct values of sorted input
Need: unique heights in descending order.
Design: `a.sorted(by: x => desc(x)).distinct()`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u19-distinct-sorted.cp`.

### seq/U53: distinct values in first-seen order, and adjacent dedup
Need: unique values keeping input order; collapse repeats.
Design: `a.distinct()` keeps first occurrences; `a.deduped()` drops adjacent repeats.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u53-dedup-unsorted.cp`.

### seq/U54: merge two sorted sequences
Need: merge step of a divide and conquer.
Design: `a.merge_sorted(b, by:)`, `kway_merge(lists)`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u54-merge-sorted.cp`, which also checks that `by:` orders the merge.

### seq/U55: union, intersection, difference of sorted lists
Need: common elements of two sorted lists.
Design: `a.sorted_union(b)`, `sorted_intersect`, `sorted_diff`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u55-sorted-set-ops-methods.cp`.

## Building, combining and editing

Make sequences, combine them, and edit them in place.

### seq/U65: build an array or list
Need: n zeros, or `a[i] = f(i)`.
Design: `List.filled(n, x)`, `[D] i => f(i)`, `List(gen)`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u65-construct.cp`.

### seq/U56: map, filter, filter_map
Need: transform and select.
Design: `a.map(f)`, `.filter(p)`, `.filter_map(f)`, lazy; `collect()` builds.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u56-map-filter.cp`.

### seq/U20: the first `Some` a function gives
Need: the first index where a sub-search succeeds.
Design: `a.first_some(f) -> ?U`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u20-first-some.cp`.

### seq/U22: flat map, flatten and concat
Need: each element expands to several.
Design: `a.flat_map(f)`, `a.flatten()`, `a + b`, `chain(a, b, ...)`; `chain` is free and variadic, like `min` (R07.7.4), and `a.chain(b)` is an error naming it.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u22-flat-map-concat.cp`; reject case `spec/conformance/diagnostics/str-chain-method-names-free-chain.cp`.

### seq/U15: walk two sequences together, or with positions
Need: pair `a[i]` with `b[i]`, or `i` with `a[i]`.
Design: `a.items`, `zip(a, b, ...)`, `zip_exact`; `zip` is free and variadic, like `min` (R07.6.4).
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u15-zip-and-items.cp`.

### seq/U57: split a list of pairs
Need: separate `(x, y)` pairs into two lists.
Design: `ps.unzip() -> (List<A>, List<B>)`.
Milestone: M1-differentiating. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u57-unzip.cp`.

### seq/U16: walk backwards and keep the keys
Need: scan from the right and still write `ans[i]`.
Design: `a.rev.items` yields `(k, x)` with `a`'s keys.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u16-rev-keeps-keys.cp`; cost program `spec/conformance/std-seq/seq-u16-rev-keeps-keys-cost.cp`; cost program `seq-u16-rev-items-cost.cp`.

### seq/U17: slices are views
Need: append the tail of one list to another without a copy.
Design: `a[l..r]` is a view; `a.extend(b[l..])` copies only what it appends.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u17-slices-are-views.cp`; cost program `spec/conformance/std-seq/seq-u17-slices-are-views-cost.cp`; cost program `seq-u17-slice-cost.cp`.

### seq/U58: insert, remove, swap, fill, reverse, rotate, resize
Need: small in-place edits.
Design: `insert(i, x)`, `remove(i)`, `swap(i, j)`, `fill(x)`, `reverse()`, `rotate_left(k)`, `resize(n, x)`, `clear()`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u58-edit-in-place.cp`.

### seq/U59: infinite and repeating generators
Need: powers of two until a bound, or a repeating pattern.
Design: `iterate(x, f)` stops at `None`; `a.cycle()` repeats.
Milestone: M1-differentiating. Status: partial
Evidence: program `spec/conformance/std-seq/seq-u59-generators.cp`; `iterate(1, x => x * 2).take(4)` overflows: `take` does not stop it; `cycle` means a functional-graph walk and needs `from:`.

## Stacks and deques

The top of a list and both ends of a deque, without `last!` or `[len - 1]`.

### seq/U21: stack top, push, pop
Need: a monotone stack.
Design: `s.push(x)`, `s.pop() -> ?T`, `s.last -> ?T`, `s.pop_while(p)`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u21-stack-top.cp`; the corpus writes `last!` in 120 ports and `[len - 1]` in 11.

### seq/U60: change the top in place
Need: replace the top of a stack.
Design: assign by index, `s[s.len - 1] = x`; `last` stays read-only, and `s.last = x` is an error whose fix writes the index form.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-cli/tests/mistakes/list_last_assign.cp` and `crates/cpl-cli/tests/mistakes/array_last_assign.cp`.

### seq/U61: push and pop at both ends
Need: a 0-1 BFS or a sliding window.
Design: `Deque`: `push`, `push_front`, `pop`, `pop_front`; a list has no `pop_front`.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u61-deque-ends.cp`; `crates/cpl-cli/tests/mistakes/list_pop_front.cp` (`List.pop_front` points at `Deque`).

## Output and shape

Turn a sequence into output, and keep one vocabulary across shapes.

### seq/U24: print or join with a separator
Need: print a list joined by `, ` or with no space.
Design: `print(a, sep: ", ")` prints any sequence; `sep.join(parts)` (Python's order) builds a `Str` from any printable sequence.
Milestone: M1-critical. Status: supported
Evidence: program `spec/conformance/std-seq/seq-u24-separator.cp`.

### seq/U25: one call works on every shape
Need: the call you know for arrays also works on a list, a slice, a `Str`, a range or a generator.
Design: every sequence fn takes `self`, so `a.f(..)` works whatever `a` is.
Milestone: M1-critical. Status: partial
Evidence: program `spec/conformance/std-seq/seq-u25-one-api-every-shape.cp`; U30, U38 and U55 now pass as methods.

### seq/U62: every call states cost, laziness and empty input
Need: know before calling whether it is \(O(n)\), lazy, or traps on empty.
Design: each std fn's doc has `cost:`, `empty:` and `traps:` lines, checked by `std-docs-lint.py`.
Milestone: M1-critical. Status: supported
Evidence: `scripts/std-docs-lint.py`, run as `--max 0` in check.sh: 0 findings.

### seq/U64: sequence features we don't build
Need: shuffling, parallel sorts, Python slices with negative steps.
Design: none: randomness lives in stress generators; `a.rev` replaces negative steps.
Milestone: later. Status: non-goal
Evidence: see the prior-art inventory.

## Changes

- 2026-11-23: 25 use cases, first version (RFC 0002 pilot).
- 2026-11-23: walked the prior art, the corpus and idioms' review: one program per use case, cost programs; positions are searches over `keys`; `ties:` on `max`/`min`; `init:` on `fold`; `print(sep:)` as the one separator.
