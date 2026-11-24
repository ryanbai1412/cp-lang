# etsr23r2p4

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against the friction's `## Dream`. The state (first unserved student, served `Subset<Offset>`, last served as `Rel`) and the two-layer sweep (lines 39-59) are the header's argument.

## Checklist failures
None beyond the friction's two gaps: the relaxes at lines 49-52 and 58 write the cell twice, and `fn at(i, j)` (line 25) turns an offset back into a student.

## Owed rewrites
None today.

## Language gaps
- No `?T` relax: `x.lower(3)` is `error[E0203]: a ?Int has no method lower`; repro now in the friction.
- No keyed window view over a domain-indexed array; in the friction.

## Dream notes
The dream streams `choices` with `yields`, relaxes with `.lower`, and indexes `queue[i + j]`; its gaps are both in the friction.
