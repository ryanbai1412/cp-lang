let n: 1..=9 = read()
domain I = 1..=n
let a: [I]Int = read()
let g: [I, I]Int = [I, I] (r, c) => r.label * 10 + c.label
let pre = prefix(a)
let sq = prefix(g)
let i = I(2)
print(pre.fold(..i), pre.fold(i..), pre.fold(..=i), pre.fold(i..=i), pre.fold(I(3)..I(2)))
print(sq.fold(..i, ..i), sq.fold(i.., i..), sq.fold(span(i, I(3)), span(I.last!, I(5))), g[I(2), I(5)] + g[I(2), I(6)] + g[I(3), I(5)] + g[I(3), I(6)])
