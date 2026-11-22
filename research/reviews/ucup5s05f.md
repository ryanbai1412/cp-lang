# ucup5s05f

Score: pending cold rescore (adopted full derivation header, `OpenNode`, `suffix_tree_edges`, `power_sums`, and §111 `and` guards; remaining gaps are `SubproductTree` and `std.strings.suffix_tree_edges` logged in friction; gated 150/150, 0 lints, 0 regressions)
Dream: 4/5

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 2/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup5s05f.md`:
- `std.poly`: `SubproductTree` / transposed multipoint evaluation (`eval_transposed`).
- `std.strings`: suffix-tree / LCP-interval edge walk (`suffix_tree_edges`).

## Cold review (HEAD)

Dream 4/5 (new), port 2 → 3/5, rescored after e0f2c521. The header derives the power-sum and transposed-evaluation route, and the program is that route.
- `SubproductTree` and `suffix_tree_edges` (lines 7, 9) are not in the stdlib; the friction reports cover them.
- Minor: `above` holds `bottom_size` and `below` holds `top_size` (lines 24-25), so the names are inverted. `inv_fact(i + 1)` (line 32) doesn't match the header's `/ k!` without explanation.

Port 3/5: it hand-writes the subproduct tree (`product`/`descend` with `2 * node` indexing) and the suffix-tree edge walk, both reported gaps.
