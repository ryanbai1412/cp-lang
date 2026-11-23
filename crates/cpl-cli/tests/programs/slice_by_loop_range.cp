// A range a comprehension, a table or a reduction slices by is the one of
// its own round, not one worked out before the loop began.
domain P = 0..3
let a: [P]Int = [6, 2, -10]
let spans: List<Range<P>> = [P.nth(0)..=P.nth(1), P.nth(2)..=P.nth(2)]
print([a[r].sum() for r in spans])
print([a[spans[k]].sum() for k in 0..2])
print([a[r.start..r.end].sum() for r in spans])
for r in spans:
    print(a[r].sum())
domain Seg = 0..2
let t: [Seg]Int = [Seg] s => a[spans[s.ord]].sum()
print(t)
print([a[r].sum() for r in spans if a[r].sum() < 5])
print(sum(a[r].sum() for r in spans))
