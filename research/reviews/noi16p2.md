# noi16p2

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`). The case split (-1, 0, 1, 2) is the header's; free spans per band joined by a `Dsu`, then cut vertices near the crickets.

## Checklist failures

- The overlap of neighbouring bands is a hand two-pointer `while` (lines 38-44) and runs are numbered by hand offsets (`top`, `bottom`, lines 33-45): the dream's `free_spans`/`overlaps`, reported.
- `walled` (lines 69-74) and the neighbourhood loops (lines 55-59) spell out squares around a cell with nested `dx`/`dy` loops.
- The final answer (line 82) packs four cases into one `if`/`elif` expression.

## Owed rewrites

- A `fn around(p, r) -> yields Cell` for both neighbourhoods.

## Language gaps

In the friction.

## Dream notes

`free_spans` and `overlaps` state the band join; the cut-vertex half is not in the dream.
