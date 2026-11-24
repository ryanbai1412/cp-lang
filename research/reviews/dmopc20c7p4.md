# dmopc20c7p4

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

Each city's count of safe children lives in a `SegTree` over preorder, so a path query is a path fold plus the LCA and its parent.

## Checklist failures

- `c + (if safe[u]: 1 else: -1)` and `Int(safe[top]) + above` do arithmetic on `Bool`s.
- `kind: 1..=2` is dispatched with `if kind == 1` and an implicit else; each kind reads its own fields inside the branch.
