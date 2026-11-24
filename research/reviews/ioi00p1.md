# ioi00p1

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 4/5. The header (lines 5-7) gives the interval recurrence and why two rows fit the 32 MB limit, and lines 15-21 are that recurrence over `I.rev` and `i.after`.
- `inner[j.prev!]` (line 19) reads the empty interval `i + 1..=i` from the zero-filled row; correct, but the header doesn't say that the zero fill is the base case.
