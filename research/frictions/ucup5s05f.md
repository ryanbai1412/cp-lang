# ucup5s05f friction report

## Missing stdlib: transposed multipoint evaluation / subproduct tree in `std.poly`
- **Where:** `proto/ports/ecnerwala/dream/ucup5s05f.cp` lines 29–30 (`proto/ports/ecnerwala/ucup5s05f.cp` lines 17–26, 35–43, 82–85).
- **What felt wrong:** `std.poly` provides forward multipoint evaluation (`eval_all`), but not a reusable `SubproductTree` or transposed multipoint evaluation (`eval_transposed` / power-sum-to-falling-factorial conversion). The active port must hand-roll a `4 * n` segment tree (`product`) and recursive `descend` with `2 * node` / `2 * node + 1` indexing.
- **Goal it hurts:** express.
- **Ideal code:**
  ```cp
  let tree = SubproductTree([Z(u) for u in 0..n], keep: n + 1)
  let ans = tree.eval_transposed(sums)
  ```
- **General fix:** Expose `SubproductTree<Z>` (or `eval_transposed`) in `std.poly`, reusing the subproduct tree already needed for `eval_all` and `interpolate`. Removes 25+ lines of `4 * n` node-arithmetic boilerplate whenever a problem needs transposed multipoint evaluation or power-sum-to-binomial conversion.
- **Severity:** medium.
- **Status:** `SubproductTree.eval_transposed` is in `std.poly` now, but it computes power sums from weights (`out[k] = sum_i c[i] * xs[i]^k`), the reverse of what this problem needs; see the transposed interpolation entry below. `power_sums` replaced the port's own.

## Missing stdlib: suffix-tree / LCP-interval walk in `std.strings`
- **Where:** `proto/ports/ecnerwala/dream/ucup5s05f.cp` lines 22–24 (`proto/ports/ecnerwala/ucup5s05f.cp` lines 45–72).
- **What felt wrong:** `std.strings` provides `suffix_array` and `lcp_array`, but walking the suffix tree's edges (or LCP intervals with `(depth, parent_depth, subtree_size)`) still requires a 25-line monotone-stack loop over `sa` and `lcp`.
- **Goal it hurts:** express.
- **Ideal code:**
  ```cp
  let edges = suffix_tree_edges(s)
  let above = [Z(e.bottom_size) for e in edges]
  let below = [Z(e.top_size) for e in edges]
  ```
- **General fix:** Add a suffix-tree / LCP-interval iterator or builder in `std.strings` yielding each edge `(parent_depth, depth, leaves)` from `suffix_array` + `lcp_array`. Removes the repeated monotone-stack suffix-tree reconstruction across string-counting problems.
- **Severity:** medium.
- **Status:** fixed: `std.strings.suffix_tree_edges` (with `top_size` / `bottom_size`) replaced the port's monotone-stack walk.

## Missing stdlib: power sums to binomial sums (transposed interpolation), or a fast `compose`
- **Where:** `corpus/problems/ucup5s05f/port.cp` `binomial_sums` (lines 14-25, the helper and its derivation).
- **What felt wrong:** from `p[j] = sum_s s^j` (j <= n) the answer is `sum_s (1 + y)^s mod y^(n + 1)`. The editorial line is `compose(egf, log([Z(1), Z(1)], n + 1), n + 1)` with `egf[j] = p[j] / j!`, but `std.poly.compose` takes 1.87 s at n = 2e5 (`cpl profile --native`; `eval_all` on the same size is 0.65 s), which puts the port at 2.1x orig.cpp. The fast route is the transpose of interpolation on the points `0..=n` (weights `w` with the same power sums, then `taylor_shift(w, 1)`), which `std.poly` lacks, so the port derives it by partial fractions in 8 lines.
- **Goal it hurts:** express, perf.
- **Ideal code:**
  ```cp
  let answer = compose(egf, log([Z(1), Z(1)], n + 1), n + 1)
  ```
  or `taylor_shift(SubproductTree(points).interpolate_transposed(p), Z(1))`.
- **General fix:** a `compose` within a small factor of `eval_all` (the O(n log^2 n) composition with a power series of the form `log(1 + y)` / Kinoshita-Li), and `SubproductTree.interpolate_transposed` (the inverse of `eval_transposed`). Either removes the helper.
- **Repro:** `use std.poly.{compose, log}` / `type Z = Mod<998_244_353>` / `let n = 200000` / `print(compose([Z(i) for i in 0..=n], log([Z(1), Z(1)], n + 1), n + 1)[1].v)`: 1.87 s native.
- **Severity:** medium.

## Submission size over the 32 KB target
- **Where:** `cpl build --judge codeforces corpus/problems/ucup5s05f/port.cp`: 40765 bytes (the previous port 36674).
- **What felt wrong:** the solution is 20 lines; the bytes are std.poly (`eval_all`, the subproduct tree, `power_sums`, `taylor_shift`, `convolve`) and `suffix_tree_edges`.
- **Goal it hurts:** dependable submission.
- **General fix:** smaller emitted std.poly code (shared NTT/tree helpers); `--size` shows where the bytes go.
- **Severity:** low.

## Known gap: `Str(len: n..=n)`
- `let s: Str(len: n..=n) of "abcdefghijklmnopqrstuvwxyz" = read()` is E0504; already in the compiler worklog (`core-02/str-len-runtime-bound.cp`). The port keeps the alphabet and drops the length.
