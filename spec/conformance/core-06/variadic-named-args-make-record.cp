// expect: ok, output "3 x 1 2"
// rule: 06.1.5
// Named arguments to a variadic `..xs` parameter make `xs` a named tuple, fields in written order (decision 323).
fn pack<Ts>(..xs: Ts) -> Ts = xs
let p = pack(n: 3, s: "x")
let q = pack(1, 2)
print(p.n, p.s, q.0, q.1)
