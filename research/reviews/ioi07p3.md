# ioi07p3

Score: 4/5
Dream: 4/5 (the friction: the port is the dream)

## Cold review (HEAD)

Port 4/5, dream 4/5. The header (lines 5-9) derives why only the per-height counts matter and why the greedy keeps them non-increasing; the loop (lines 17-24) is that argument, with a range-add min `SegTree` and two `right_while`s.
- `k` is read as `1..=1e5`, not `1..=h` (line 13); in the friction.
- `Height.cuts(Int(level) + rest)` and the `Int(top) - Int(below)` count (lines 23-24) go through `Int`; in the friction.
