# ioi05p4

Score: 4/5
Dream: 4/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-9) derives the shift argument and the `(n - gap + 1) / 2` answer, and `worst` (lines 25-29) is it: sort the shifts, take the largest circular gap, and try both directions (line 32).
- `wrap` plus the in-list gaps (lines 27-28) is one circular-gaps fact in two terms; in the friction (`circular_gaps(period)`).
- `random_wish` (lines 17-19) exists to generate a permutation; in the friction (`Perm<Seat>`).
- The friction's "window slices keep the list's keys" entry names `w.last! - w.first!`, which the port no longer has (line 28 indexes `shifts[j - 1]`).
