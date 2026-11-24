# mccc4j4

Score: 2/5
Dream: 3/5

Cold review: correct, but the reads, the output and the result dispatch are C++ bookkeeping that today's language removes, and none of it is reported.

## Checklist failures

- R5: `let (n, m): (Int, Int)` (line 11) and `let k: Int` (line 32) take no statement bounds; the `// Bounds:` line names only `t, u, v`.
- R3: the groups are read by a nested push loop (lines 15-19); `read<[n / m][m]Team>()` reads them (a `cpl run` of `read<[n / m][m]P>()` works today), as dream.cp line 5 does.
- R3: lines 38-42 join `out` with spaces by hand; `print(out)` prints a list space-separated.
- R2: `if res == 'W' ... elif ... else` (lines 24-30) where dream.cp's `match res:` names the three results; `groups[g].ord + 1` (line 36) prints a `Team` through its 0-based rank where the team prints itself.

## Owed rewrites

- Read the groups and the results with typed reads, `match` on the result, and `print(groups.map(g => List(g).sorted(by: t => (desc(pts[t]), t))[k - 1]))` as in dream.cp line 15.

## Language gaps

- `groups.rows` (E0203) is in research/frictions/mccc4j4.md; nothing else here needs the language.

## Dream notes

- dream.cp also reads `(n, m): (Int, Int)` and `k: Int` (lines 3 and 14): the bounds are untyped there too. Otherwise it is the editorial.
