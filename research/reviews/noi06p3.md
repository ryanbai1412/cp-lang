# noi06p3

Score: 2/5
Dream: n/a

Scored cold against the checklist, the dream and the port.

The port reads typed rows and splits them into `a` and `b`, then runs a two-pointer DP over at most 25 candidate heights per column, twice. The reads and the `?Int` costs are at bar; the DP is still the C++'s shape. It agrees with the v0.2 port on 30 stress cases and passes 8 package cases.

## Checklist failures

- `cands_at(arr, i)` and `solve_env` index by `Int` and convert with `Col(i)`, `Col(i + 1)` and `Col(j)` throughout, so the `Col` domain buys nothing in the DP.
- `dp_lo`/`dp_hi` and `new_lo`/`new_hi` are parallel `List<?Int>`s over the candidate list, with cursors `q` walked by `while q < m_nxt` and `while q >= 0`. A merge of two sorted candidate lists is hand-written twice, once per direction.
- The candidate window `max(0, i - 2)..=min(n - 1, i + 2)` and `aj - 2..=aj + 2` clamp by hand where a window over the column domain would clip itself.

## Gaps that need the language

- The dream's state space does not fit the bounds: research/frictions/noi06p3.md.
