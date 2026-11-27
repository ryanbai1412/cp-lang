// expect: ok, output "70 4\n264 2\n4 3 0\n3 2"
// rule: R04.4.25 R04.4.8 R07.4.6
// An exclusive literal end where positions are wanted is a cut, so it may be the domain's end:
// `g[1..3, 2..4]` and `a[..4]` over a 4-position domain reach its last position.
domain R = 0..4
domain C = 0..4
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 10 + Int(c)
print(g[1..3, 2..4].sum(), g[1..3, 2..4].len)
print(g[.., ..4].sum(), g[3.., 2..].len)
let a: [C]Int = [1, 2, 3, 4]
print(a[..4].len, a[1..4].len, a[..0].len)
let r: Range<C> = 1..4
print(r.len, r[2..4].len)
