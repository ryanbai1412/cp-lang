# dmopc20c4p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

A sweep from the top: cords and roombas sorted by descending y, standing cords counted per x in a `Fenwick` over the compressed xs, and each roomba adds the count at or left of its x.

## Checklist failures

- `while added < m and cords[added].y >= r.y` walks a manual cursor into the sorted cords; a merge of the two sorted sequences would state it.
- `xs.index_of(cords[added].x)` looks up each cord's compressed index again; `compress` could hand back the index per cord.
- `if let reach = xs.last_le(r.x): touching += standing.prefix(reach.r)` mixes a position and a cut (`reach.r`) to say "at or left of x".
