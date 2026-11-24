# btoi17p3

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction)

Cold review of `port.cp`, without the C++. Each minister's cities are sorted by preorder
(`plan.sorted(by: tree.pos)`) and walked cyclically, marking +1 at each leg's start and -1 at
its `lca`. The children-first sum then counts the ministers per track: the editorial, with
`City` and `Track` as domains. With no dream the port scores at most 3.

## Checklist failures
- idiomatic-code/R3: `fn lower(u, v)` (line 30) re-derives std `Rooted.lower(u, v)` (the
  endpoint farther from the root). Unreported.
- The children-first fold (lines 27-28) is a hand subtree sum; fine today.

## Owed rewrites
- `marks[tree.lower(tracks[t].u, tracks[t].v)]` and drop `fn lower`.
- Write a dream.

## Language gaps
None; the friction reports none.
