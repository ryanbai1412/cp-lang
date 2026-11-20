// `prefix` of `Mod`s and `F64`s, as of `Int`s, in one and two dimensions.
type Z = Mod<998244353>
let n = 5
domain I = 0..n
let a: [I]Z = [I] i => Z(i.to_idx + 1) * 998244352
let p = prefix(a)
print(p.sum(I(1)..=I(3)), p.sum(I(2)..I(2)))
let b: [I]Int = [I] i => i.to_idx * 2
print(prefix(b).sum(I(0)..=I(4)))
let f: [I]F64 = [I] i => 0.5
print(prefix(f).sum(I(1)..=I(4)))
let g: [I, I]Z = [I, I] (r, c) => Z(r.to_idx * c.to_idx)
print(prefix(g).sum(I(1)..=I(4), I(2)..=I(3)))
let xs = [3, 1, 4, 1, 5]
let q = prefix(xs)
print(q.sum(1..=3), q.sum(0..5), q.sum(2..2), q.prefix(2))
var ys: [n]Int = 0
for i in 0..n: ys[i] = i * i
let r = prefix(ys)
print(r.sum(1..=3), r.sum(0..n))
let zs: List<Z> = [Z(1), Z(2), Z(998244352)]
print(prefix(zs).sum(0..3))
