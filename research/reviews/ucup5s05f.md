# ucup5s05f

Score: 2/5

The header is a real editorial (power sums by logarithmic derivative, transposed multipoint evaluation), but the product tree and the suffix-tree walk are hand-rolled with node arithmetic and sentinels. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Hand-rolled standard algorithms (lines 11-39): subproduct tree over `4 * len` with `2 * node`, and the transposed evaluation descent.
- Hand stack (lines 50-51, 57): fixed array plus `open_len` instead of a `List`.
- Sentinels and `while true` (lines 54-58): `-1` for "no next lcp" and "empty stack".

## Owed rewrites

- Use `List` as the stack with `last`; give the suffix-tree walk a named iterator or file it.

## Language gaps

No friction report exists. Missing: `poly` subproduct tree / multipoint evaluation, a suffix-tree (lcp-interval) walk in `std.strings`.
