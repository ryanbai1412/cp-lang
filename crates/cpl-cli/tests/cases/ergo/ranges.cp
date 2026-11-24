let n: 1..=9 = read()
domain I = 1..=n
type B = I.cuts
let a: [I]Int = read()
fn total(rows: Range<I>) -> Int: sum(a[i] for i in rows)
let i = I(3)
print(total(..i), total(..=i), total(i..), total(i.r..), total(..i.l))
print(sum(a[x] for x in ..i), [x for x in i..], [x for x in i.l..i.r])
for b in I.cuts:
    print(b, total(..b), total(b..), total(span(b, i)), total(span(i, b)), span(b, I.cuts.last!).len)
print(total(span(I(2), I(5))), total(span(I(5), I(2))), sum(a[i.r..]), sum(a[..=i]), a[..i].max()!)
for x in ..i: print(x)
for x in (i..).rev: print(x)
for k in ..3: print(k)
print(yes(i in ..i), yes(i in ..=i))
