# ioi07p4

Score: 3/5
Dream: 4/5

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-9) gives the backward DP over both mines' last two foods, and lines 24-28 are it. The state is packed by hand: `Mine = 0..16` as `older * 4 + old` (lines 17-21) and the foods coded 1..=3 by an `if` chain (line 26).
- Both are forced by gaps in the friction: `[?Food]` as an index (E0204) and reading an enum from its letter (E0302); the runs are now quoted there.
