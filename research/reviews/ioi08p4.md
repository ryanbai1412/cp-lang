# ioi08p4

Score: 3/5
Dream: 4/5

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-10) derives the ride-plus-loops structure and the greedy (longest loops first, then crossing pairs), and lines 54-63 are that greedy. The graph part is bookkeeping:
- `owner`, `first_end` and the `next` counter (lines 23-38) number the endpoints west to east by hand; in the friction (coordinate compression as a domain).
- `ride` walks the stretches with a shared `seen` over raw `0..=2 * n` (lines 44-52); in the friction (the ride is a path, so `Functional` doesn't fit).
