# ioi09p8

Score: 4/5
Dream: 4/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 4/5, from the friction's `## Dream`. The header (lines 7-11) derives the two max `SegTree`s and the per-day monotone sweeps, and `reach`, `stand` and the day loop (lines 32-54) are exactly that.
- The two carried sweeps (lines 46-53) are index loops over a per-day list; in the friction (`scan` folds same-typed values only).
