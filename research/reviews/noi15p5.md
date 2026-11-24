# noi15p5

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. Suffix array and LCP, adjacent ranks united from the largest LCP down, each group keeping size and extreme scores (lines 30-47), as in the header.

## Checklist failures

- The friction says the pairs are bucketed by LCP (`meet_at[r]`) "instead of moving a pointer through a sorted list", but the port sorts by LCP and moves `next` through it (lines 40-46). The friction is corrected.
- `join` (lines 30-37) updates `lo`/`hi` beside the `Dsu` by hand.

## Owed rewrites

- Add a `## Dream`.

## Language gaps

None.

## Dream notes

None.
