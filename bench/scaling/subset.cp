// n: 98304
let n: Int = read()
domain B = 0..6
var s = Subset<B>.empty
var acc = 0
for i in 0..n:
    let b = B(i % 6)
    var t = Subset<B>.single(b) // timed: subset.Subset.single
    s = s.with(B(i * 7 % 6)) // timed: subset.Subset.with
    s = s.without(B(i * 3 % 6)) // timed: subset.Subset.without
    t.set(B(i * 11 % 6), true) // timed: subset.Subset.set
    if s.contains(b): acc += 1 // timed: subset.Subset.contains
    acc += s.len // timed: subset.Subset.len
    if s.is_empty: acc += 1 // timed: subset.Subset.is_empty
    acc += s.shifted(i % 3 - 1).len // timed: subset.Subset.shifted
    acc += s.complement().len // timed: subset.Subset.complement
    acc += (s.first() ?? B(0)).ord // timed: subset.Subset.first
    acc += Subset<B>.empty.len + Subset<B>.full.len // timed: subset.Subset.empty
    acc += Subset<B>.full.len // timed: subset.Subset.full
    for x in s.items: acc += x.ord // timed: subset.Subset.items
    for x in s.each(): acc += x.ord // timed: subset.Subset.each
    let u = Subset<B>.single(B(i % 6)).with(B(i * 5 % 6)).with(B(i * 13 % 6))
    for g in u.subsets: acc += g.len // timed: subset.Subset.subsets
    for g in u.complement().complement().with(B(0)).complement().supersets: acc ^= g.len // timed: subset.Subset.supersets
    acc += t.len
print(acc)
