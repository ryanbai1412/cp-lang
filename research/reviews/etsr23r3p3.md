# etsr23r3p3

Score: 4/5
Dream: 5/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 5/5. The dream is the editorial in eleven lines: std `dijkstra` for the walk home, std `kruskal_tree` by descending altitude, `kt.up(min)` for the least walk per subtree and `kt.reach(v, desc(p + 1))` for the car's reach (dream lines 8-16). The port's body (lines 21-34) is the same program.
- Lines 28-30 read each query inside `for _ in 0..q:` as two `let`s. The dream reads the queries as one typed list, `for (v0, p0) in read<[q](V, 0..=s)>()` (dream line 13); the forced-online decode only needs `last`, so reading up front is the same program, and the port owes it.
- Lines 31-32 decode the online shift by hand (`V.nth((v0.ord + k * last) % n)`). `V.ring.step` is the doc's form and is E0501 today (in the friction, H-RING-RECEIVER).
- `var last` (line 27) carries the last answer between queries; that is the statement's forced-online rule, not hidden state.
