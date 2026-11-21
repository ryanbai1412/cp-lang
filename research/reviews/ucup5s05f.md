# ucup5s05f

Score: 2/5

Re-reviewed cold on main 0f1ea86c after the `prod_linear` migration: score unchanged. `power_sums` now uses `prod_linear` (line 26). Line 27 reverses and pads it by hand with `m[m.len - 1 - i]` and `Z(0)`; write `m.reverse().resized(deg + 1, 0)`. The hand `product` segment tree (lines 14-20) is still built for the transposed evaluation (lines 80-82), with `4 * n` sizing and `2 * node` arithmetic. `eval_all` (`stdlib/poly.cp:302`) is the forward evaluation, so transposed multipoint evaluation is a genuine stdlib gap. No friction report names it; flag it.

The header is a real editorial (power sums by logarithmic derivative, transposed multipoint evaluation), but the product tree and the suffix-tree walk are hand-rolled with node arithmetic and sentinels. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Hand-rolled standard algorithms (lines 11-39): subproduct tree over `4 * len` with `2 * node`, and the transposed evaluation descent.
- Hand stack (lines 50-51, 57): fixed array plus `open_len` instead of a `List`.
- Sentinels and `while true` (lines 54-58): `-1` for "no next lcp" and "empty stack".

## Owed rewrites

- Use `List` as the stack with `last`; give the suffix-tree walk a named iterator or file it.

## Language gaps

No friction report exists. Missing: `poly` subproduct tree / multipoint evaluation, a suffix-tree (lcp-interval) walk in `std.strings`.

Re-checked after 3d2276cc moved the imports to `std.ntt`/`std.poly`; the score stands, as the failures above are in the body.
