# mccc5j4

Score: 4/5
Dream: 3/5

Cold review: the feasibility test w <= r <= 26w, then the smallest feasible letter per asterisk (lines 16-27), exactly the header. Port score provisional until the dream is rewritten.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- Letter values by `Char` arithmetic, and the in-place fill with two `var`s: both in research/frictions/mccc5j4.md.

## Dream notes

- `s.fill_stars(greedy)` names the greedy without writing it, and `wild` is never defined; the dream should write the `max(1, left - 26 * rest)` choice as a scan.
