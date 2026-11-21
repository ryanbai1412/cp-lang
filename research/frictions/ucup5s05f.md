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
