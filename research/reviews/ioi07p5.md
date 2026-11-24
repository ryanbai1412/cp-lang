# ioi07p5

Score: 3/5
Dream: 4/5

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-12) derives the 45-degree turn and the three cases, and `line`, `grid` and `cube` (lines 18-57) follow it.
- The clipped box `(x.prev(r) ?? V.first!)..=(x.next(r) ?? V.last!)` is written three times (lines 37, 55, 56); in the friction (no clipped `within`).
- `line` and `grid` each keep a hand two-pointer (`while ... from += 1`, lines 23 and 34-36) for "the window within d to the left"; not in the friction.
