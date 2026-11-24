# ioi18p3 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi18p3.md).

Reads like the editorial: two std `kruskal_tree`s, each trip's human and wolf subtrees, and an
offline sweep over the human preorder with a `Fenwick` over the wolf preorder.

## Checklist failures

- `var wolf_range: [Trip]Range<wt.Pre> = [Trip] _ => wt.sub(wolf.leaf[V.first!])` (line 33)
  fills a placeholder overwritten for every trip on line 37; build it as
  `[Trip] i => wt.sub(wolf.reach(trips[i].e, Int(trips[i].r)))` (R4).
- The rectangle query as signed `asks` events (lines 32-47) is the sweep the header names; the
  dream's `.meets` would hide it, but no std 2D offline count exists.

## Language gaps

- In research/frictions/ioi18p3.md (`kruskal_tree` landed from this port).

## Dream notes

- At bar: two trees and one `meets` line.
