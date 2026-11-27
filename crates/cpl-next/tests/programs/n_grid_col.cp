// expect: ok, output "1 63 21\n41 50\n2 30\n2"
// rule: R04.4.11
// `g[.., c]` is column `c`, keyed by the row domain; `g[r, ..]` and `g[rs, c]` are pieces keyed
// by the grid's positions; `l..r` with position ends, of a declared domain or of a parameter
// used as one, is the range of those positions.
domain R = 1..=3
domain C = 0..2
let g: [R, C]Int = [R, C] (r, c) => 10 * Int(r) + Int(c)
let col = g[.., C(1)]
print(col.keys.first!, col.sum(), col[R(2)])
print(g[R(2), ..].sum(), g[R(2)..=R(3), C(0)].sum())
print((R(1)..R(3)).len, g[R(3)][C(0)..C(1)].sum())
fn span<D: Domain>(a: D, b: D) -> Int = (a..b).len
print(span(R(1), R(3)))
