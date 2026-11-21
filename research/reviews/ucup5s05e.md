# ucup5s05e

Score: pending (was 3/5: replaced `rm` loop with `(0..hull.len).max(by:)`, replaced `near` while-loop with clamped `max`/`min`, replaced `(i + 1) % 4` with `quad_edges.all(...)`, extracted `off_both_diags`, added dream + friction for `combinations(k)`)
Dream: pending (added dream using `cands.combinations(3).first(...)`)

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 3/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup5s05e.md`:
- `std.seq`: `combinations(k)` for iterating `k`-element subsets of a list.
