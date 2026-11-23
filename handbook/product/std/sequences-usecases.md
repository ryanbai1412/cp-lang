# Sequences: use cases

| | |
|---|---|
| Kind | Use cases |
| Owner | stdlib |
| Surface | [sequences.md](sequences.md) |

Each entry is one need. The ID names the need, so a design change keeps the
ID. Probes are short programs run against main on 2026-11-23; supported
needs are proven by the program case named under Evidence, which runs in CI.

### seq/U1: value of the first or last match

- **Need:** you need the element itself, e.g. the first value over a threshold.
- **Design:** `a.find(p) -> ?T`, `a.rfind(p) -> ?T`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U2: position of the first or last match

- **Need:** you need where a property starts to hold, in the array's own index space.
- **Design:** dream: `a.position(p) -> ?I`; today `a.keys.find(i => p(a[i])) -> ?I`.
- **Milestone:** M1-critical. **Status:** partial.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp` (the `keys` form); `a.find` on `[I]T` gives `?Int`, E0302 against `?I` (probe u01b).

### seq/U3: position of the extreme, with a tie rule

- **Need:** argmax of a score, where the problem says "the leftmost", "the rightmost" or "the widest".
- **Design:** `a.argmax(by:, ties: first | last) -> ?I`; widest is `runs` over the maxima.
- **Milestone:** M1-critical. **Status:** partial.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp` (first tie); `ties:` is E0310 (probe u02).

### seq/U4: extreme value, possibly empty

- **Need:** the max of a list that may be empty, or by a key.
- **Design:** `a.max(by:) -> ?T`, `a.min(by:) -> ?T`; `?? x` names the fallback.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U5: transition point of a monotone predicate

- **Need:** binary search on the answer.
- **Design:** `(lo..hi).first_true(p) -> ?T`, `.last_true(p)`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U6: first element at least `x` in sorted data

- **Need:** `lower_bound` on a sorted array.
- **Design:** `a.first_ge(x) -> ?T`, `first_gt`, `last_le`, `last_lt`; positions through `keys`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U7: range folds from one prefix table

- **Need:** sums of `a[l..r]` in \(O(1)\) after \(O(n)\).
- **Design:** `a.scan(+)` then `.fold(l..r)`, `.prefix(c)`. A table from another op answers prefix reads, and a range read from it is an error; today that error is a trap, and the goal is a compile error.
- **Milestone:** M1-critical. **Status:** partial.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp` (`+`); trap case `spec/conformance/std-prefix/scan-min-range-fold-traps.cp`.

### seq/U8: prefix folds over a slice

- **Need:** the same table over part of the array.
- **Design:** `a[l..r].scan(+)`, keyed by the slice's cuts.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U9: a fold with a seed or a custom step

- **Need:** count something with state as you walk.
- **Design:** `a.fold((acc, x) => .., init)`.
- **Milestone:** M1-critical. **Status:** gap.
- **Evidence:** `[3, 1, 4].fold((s, x) => s + x, 10)` is E0203 in `cpl run` (probe y-fold); the std fn exists.

### seq/U10: runs of equal values

- **Need:** compress `aabccc` into blocks, with their lengths.
- **Design:** `a.runs()` yields each run as a view; also on `Str`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`; probe u20 (`Str`).

### seq/U11: group by a key, in a defined order

- **Need:** bucket values by `x % k`.
- **Design:** `a.group(by:)` in ascending key order.
- **Milestone:** M1-differentiating. **Status:** supported.
- **Evidence:** probe z-grouporder (ascending keys).

### seq/U12: sort or sorted by key, descending, stable

- **Need:** order events by time, ties by input order.
- **Design:** `a.sort(by:, desc:)`, `a.sorted(by:, desc:)`, stable.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U13: the order of a sort, and its inverse

- **Need:** "the rank of each element" or "which index is the i-th smallest".
- **Design:** `a.keys.sorted(by: i => a[i])`; `p.inverse()` on a permutation.
- **Milestone:** M1-critical. **Status:** partial.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp` (sorted keys); `perm_order` is E0203 on `[I]Int` (probe x-permArr).

### seq/U14: fixed windows and adjacent pairs

- **Need:** sum of each length-`k` window, or each difference.
- **Design:** `a.windows(k)`, `a.adjacent()` as views.
- **Milestone:** M1-critical. **Status:** partial.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`; no allocation check yet (`std-api/R17`).

### seq/U15: walk two sequences together

- **Need:** pair `a[i]` with `b[i]`.
- **Design:** `zip(a, b)`, `a.zip(b)`.
- **Milestone:** M1-critical. **Status:** gap.
- **Evidence:** `a.zip(b)` on `[I]Int` is E0203 in `cpl run` (probe x-zipArr).

### seq/U16: walk backwards and keep the keys

- **Need:** scan from the right and still write `ans[i]`.
- **Design:** `a.rev.items` yields `(k, x)` with `a`'s keys.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U17: slices are views

- **Need:** append the tail of one list to another without a copy.
- **Design:** `a.extend(b[l..])`; `a[l..r]` is a view.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U18: count, sum, any, all, contains on any shape

- **Need:** the one-line checks.
- **Design:** `a.count(p)`, `.sum()`, `.any(p)`, `.all(p)`, `.contains(x)`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U19: distinct values of sorted input

- **Need:** unique heights in descending order.
- **Design:** `a.sorted(desc: true).distinct()`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U20: the first `Some` a function gives

- **Need:** the first index where a sub-search succeeds.
- **Design:** `a.first_some(f) -> ?U`.
- **Milestone:** M1-differentiating. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U21: stack top, push, pop

- **Need:** a monotone stack.
- **Design:** `s.push(x)`, `s.pop() -> ?T`, `s.last -> ?T`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U22: flat map and concat

- **Need:** each element expands to several.
- **Design:** `a.flat_map(f)`, `a.chain(b)`.
- **Milestone:** M1-differentiating. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U23: running extremes

- **Need:** the max so far at each position.
- **Design:** `a.scan(max)` read with `.prefix(c)` and `.all`.
- **Milestone:** M1-critical. **Status:** supported.
- **Evidence:** program `spec/conformance/std-seq/product-sequences-supported.cp`.

### seq/U24: print or join with a separator

- **Need:** print a list joined by `, ` or with no space.
- **Design:** `print(a, sep: ", ")`; `a.join(sep)` on `Str`s.
- **Milestone:** M1-critical. **Status:** partial.
- **Evidence:** `[1, 2, 3].join(", ")` is E0302 (probe z-join); `print(a, sep:)` works.

### seq/U25: one call works on every shape

- **Need:** the call you know for arrays also works on a list, a slice, a `Str` or a generator.
- **Design:** the same std fn over `Seq`/`Indexed`, no shape-specific names.
- **Milestone:** M1-critical. **Status:** partial.
- **Evidence:** U9, U13 and U15 fail on lists or arrays while the std fn exists.

## Changes

- 2026-11-23: first version, 25 use cases (RFC 0002 pilot).
