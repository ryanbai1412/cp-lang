# agc077a

Score: 2/5

The `Seg` record is a good idea, but the body carries two hand prefix-sum arrays with duplicated getters, raw `0..n` positions and `+ 1` output shifts. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Duplicated formula (lines 36-37): `get_px` and `get_py` are the same function over two arrays.
- Hand prefix sums (lines 30-34): `xs.map(c => Int(c == 'B')).prefix_sums()`.
- Raw index space: `n: Int` (line 11) without a domain; output shift `a + 1, b + 1` (line 65).
- Header (lines 1-2) names the procedure, not the invariant that the binary search finds.

## Owed rewrites

- One `count_b(s: Str, seg: Seg)` over named prefix sums; `domain Pos`; state the invariant.

## Language gaps

No friction report exists.
