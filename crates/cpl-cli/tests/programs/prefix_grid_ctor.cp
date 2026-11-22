// `Prefix(a)` on an `[R, C]` array infers the 2-D shape, summed over rows and cols.
domain R = 0..2
domain C = 0..3
let p = Prefix([R, C] (r, c) => 1 + 3 * Int(r) + Int(c))
print(p.sum(R(0)..=R(1), C(0)..=C(2)), p.sum(R(1)..=R(1), C(0)..=C(0)))
domain D = 0..4
let q = Prefix([D] v => 1 + Int(v))
print(q.sum(D(1)..D(3)))
