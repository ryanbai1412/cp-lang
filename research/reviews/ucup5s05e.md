# ucup5s05e

Score: 3/5
Dream: 3/5

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 3/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup5s05e.md`:
- `std.seq`: `combinations(k)` for iterating `k`-element subsets of a list.

## Cold review (HEAD)

Dream 3/5, port → 3/5, rescored on current main; sent back.
- Owed: the header asserts that only a few hull vertices around the query's x can matter (lines 3-5), and `near` hard-codes `at + 4`/`hi - 5` (lines 35-36). Derive why five chain vertices on each side, plus one off-diagonal point, suffice; that is the solution's invariant.
- Owed: `quad_edges` (lines 27-28) lists the cyclic hull pairs by hand.
- Reported gap: `combinations(3)` (line 43; the `t[0]` E0305 follows from it).
- Minor: `for _ in 0..q` (line 51) should be `repeat q`.

Port 3/5: a triple loop in place of `combinations` (reported).
