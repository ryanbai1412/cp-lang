# ioi00p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction)

With no dream (no dream.cp and no `## Dream` in the friction) the port scores at most 3 (README); against the checklist alone it reads as a 4.

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 4/5. The header (lines 5-7) gives the interval recurrence and why two rows fit the 32 MB limit, and lines 15-21 are that recurrence over `I.rev` and `i.after`.
- `inner[j.prev!]` (line 19) reads the empty interval `i + 1..=i` from the zero-filled row; correct, but the header doesn't say that the zero fill is the base case.
