# csps21p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

An interval DP over `[Pos, Pos]` tables: `wrap[l, r]` counts sequences closed by a bracket pair at l and r, `seq` the full count over l..=r, and `tail` the joins with a trailing star run. Star runs are checked through one `Prefix` of non-star cells.

## Checklist failures

- `wrap`, `seq` and `tail` are three parallel `[Pos, Pos]Z` tables filled in one loop, which is the recurrence's shape but nothing ties them together.
- `seq[i, r.prev!]` and `seq[l.next!, i]` unwrap neighbours that the loop bounds guarantee, so the types do not carry the proof.
- `stars(l.r..i.l)` and `stars(i.r..r.l)` build star runs from cut arithmetic; positions read as cuts here, which is the gap in the friction report.

## Gaps that need the language

- A cut range iterating as cuts where positions are meant: research/frictions/csps21p2.md.
