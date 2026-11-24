# ioi14p3 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi14p3.md).

Reads like the editorial: a std `Dsu`, asked counts once per unordered pair in
`[Pair<V>]Int`, "yes only to the last unasked pair between two components", and the kept
root's counts folded in on merge. The `ask("READY")`-then-reply threading is reported.

## Checklist failures

- `answer` returns `Int` 0/1 (the protocol's encoding) where the rule is a `Bool`; minor.

## Language gaps

- A callback-style interactive entry: in research/frictions/ioi14p3.md.

## Dream notes

- `serve(fn(u0, v0) -> Int: ...)` is the port's rule with the protocol gone; at bar apart from
  the 0/1 reply.
