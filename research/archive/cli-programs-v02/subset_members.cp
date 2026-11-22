// `for i in mask` over a `Subset<D>`: the members, lowest first, forward,
// reversed and in a comprehension.
let n: 1..=10 = read()
domain I = 0..n
let a: [I]Int = read()

var best: [Subset<I>]Int = 0
for s in Subset<I>:
    for i in Subset<I>.full - s:
        best[s.with(i)] max= best[s] + a[i]
print(best[Subset<I>.full])

let odd = Subset<I>.full - Subset<I>.empty.with(I.first)
for i in odd: print(i.idx)
for i in odd.rev: print(i.idx)
print(sum(a[i] for i in odd))
