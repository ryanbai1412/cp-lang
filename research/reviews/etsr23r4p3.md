# etsr23r4p3

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 4/5. The header (lines 6-10) derives the subset tree DP, the inclusion-exclusion sign and why wrapping `U64` sums stay exact. The body is that argument: `Subset<V>` adjacency, a children-first walk over std `Rooted` (`tree.at.rev`, line 31), and the signed sum (line 38).

## Owed rewrites

- Lines 19-23 build the `Subset` adjacency with a read loop; reading `[m](V, V)` and folding it would match how `tree_edges` is read on line 24.
