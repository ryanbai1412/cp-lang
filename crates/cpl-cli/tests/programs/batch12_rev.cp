// Batch #12: a reversed range kept as a value is the list, last first.
let n: Int = read()
domain D = 0..n
fn total(xs: List<D>) -> Int: sum(x.ord for x in xs)
for i in D.all.rev: print(i, end: " ")
print()
let back: List<D> = D.all.rev
let r = D.all.rev
print(back, total(D.all.rev), [i.ord * 2 for i in D.all.rev], [i for i in r])
print([i for i in (1..=n).rev], sorted(D.all.rev, by: i => i.ord))
print(D.all.rev(), (0..n).rev().len)
