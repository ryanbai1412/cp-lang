# dmopc20c7p4

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Each city's count of safe children lives in a `SegTree` over preorder, so a path query is a path fold plus the LCA and its parent.

## Checklist failures

- `c + (if safe[u]: 1 else: -1)` and `Int(safe[top]) + above` do arithmetic on `Bool`s.
- `kind: 1..=2` is dispatched with `if kind == 1` and an implicit else; each kind reads its own fields inside the branch.
