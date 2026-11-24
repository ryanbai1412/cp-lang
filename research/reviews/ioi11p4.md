# ioi11p4 review

Score: 3/5
Dream: none (no dream.cp and no `## Dream` section; the friction report says the port is the dream)

Scored cold against the checklist and the port. A port with no dream scores at most 3.

The port itself is close to the editorial: typed reads (`Room`, `Hall`, `X`), a std
`Graph.undirected` and `MinHeap`, and the "settle on the second pop" Dijkstra the header
derives. It is capped by the missing dream, and the friction report had no entry for the
grader task the header names.

## Checklist failures

- `offers[x] = 1` for an exit (line 24) seeds the counter so an exit settles on its first pop:
  a trick the comment states but that reads as a sentinel; a `need: [Room]0..=2` (2, or 1 for
  an exit) would say the rule.
- `if offers[u] != 2: continue` then `if u == Room.first: print(t); break` checks the start
  inside the loop; fine, but the answer is printed from the middle of the search.

## Language gaps

- The judge's function task (`travel_plan`) has no CP Lang form: added to
  research/frictions/ioi11p4.md (same E0501 as ioi14p1).

## Dream notes

- None written. The friction report should carry a `## Dream` (the `@grader fn travel_plan`
  form, a `second_best` Dijkstra spelled as the editorial's "worth = second-best offer").
