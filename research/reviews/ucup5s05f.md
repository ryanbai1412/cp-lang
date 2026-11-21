# ucup5s05f

Score: pending (was 2/5: added dream + frictions for `std.poly.SubproductTree`/`eval_transposed` and `std.strings.suffix_tree_edges`, replaced hand reverse-pad in `power_sums` with `prod_linear(sizes).rev.to_list().resized(deg + 1, Z(0))`, extracted `suffix_tree_edges(s)` using `List<OpenNode>` with `open.last` / `pop()` eliminating `open_len`, `-1` sentinels, and `while true`, bounded `n: 1..=2e5`)
Dream: pending (added dream using `SubproductTree.eval_transposed`, `suffix_tree_edges(s)`, and `prod_linear(...).rev.to_list().resized(...)`)

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 2/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup5s05f.md`:
- `std.poly`: `SubproductTree` / transposed multipoint evaluation (`eval_transposed`).
- `std.strings`: suffix-tree / LCP-interval edge walk (`suffix_tree_edges`).
