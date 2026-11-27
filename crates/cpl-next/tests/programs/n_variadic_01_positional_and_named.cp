// expect: ok, output "1 2 x\n3 y\n4 0\n5 9"
// rule: 06.1.5
// The arguments past the others are one tuple when positional and one record when named, in written order.
fn pack<Ts>(..xs: Ts) -> Ts = xs
fn after<Ts>(a: Int, ..xs: Ts) -> Ts = xs
fn size<Ts>(a: Int, ..xs: Ts) -> Int = a
let p = pack(1, 2, "x")
let r = pack(n: 3, s: "y")
let t = after(0, 4)
print(p.0, p.1, p.2)
print(r.n, r.s)
print(t.0, size(0))
let q = after(a: 1, lo: 5, hi: 9)
print(q.lo, q.hi)
