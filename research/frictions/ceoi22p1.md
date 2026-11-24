# ceoi22p1 frictions

## Dream

The committed `dream.cp` is the target. The port differs where the language moved on:
`deck.next_smaller(by: c => -c.ord)` for the removed `next_greater`, `Q.collect().sorted(by:)`
for a comprehension over the domain, and `Pos.first!`/`Pos.last!` (they are optional now).

## Next greater through a negated key

Next greater is spelled as the next smaller under `-c.ord`; a `next_larger` (or `next_smaller`
with `order: desc`) would say it directly.
