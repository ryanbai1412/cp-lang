# ioi14p2 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi14p2.md).

Reads like the editorial: a clamp `(lo, hi)` composes into a clamp, a lazy SegTree action
applies each operation over `l..=r`, and each column is read once. `TALLEST` is written once.
The hand-written clamp action and the grader task are reported.

## Checklist failures

- `SegTree(max, ...)` carries a `max` fold that is never queried; only `apply` and `get` are
  used. std has no dual (update-only) segment tree today, so this rides on the reported clamp
  gap.

## Language gaps

- A std clamp action and the grader fn's output array: in research/frictions/ioi14p2.md.

## Dream notes

- `Clamp.action` and `Clamp.of(op, h)` name the object; the dream is the whole grader body.
