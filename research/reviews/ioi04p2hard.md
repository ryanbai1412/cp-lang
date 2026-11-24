# ioi04p2hard

Score: 2/5
Dream: 3/5

## Cold review (HEAD)

Port 2/5, dream 3/5, from the friction's `## Dream`. The backward DP over walk prefixes with prefix-min staircases is the header's plan (lines 6-10), and the staircase over a `Map` is measured against a SegTree (2.0 s vs 0.54 s, in the friction).
- `offer_x` and `offer_y` (lines 41-51) are one function written twice with x and y swapped, and lines 58-63 write the same three-line update twice (idiomatic-code/R4). One `fn offer(axis)` and one update over an axis pair say each once; this is not in the friction.
- `insert`/`upto` (lines 27-34) are free functions threading a raw `Map` through four globals; a `struct Staircase` with those methods names the invariant (lines 23-24) today, even before std has one.
- `stop[if i > 0: i - 1 else: 0]` (line 57) and the glued stop 0 (line 15); the latter is in the friction.

## Dream notes

- Two lines naming `Staircase(min)`; the axis duplication is not addressed.
