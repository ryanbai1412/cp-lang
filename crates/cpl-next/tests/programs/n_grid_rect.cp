// expect: ok, output "4 1\n38 4\n91004\n2"
// rule: R04a.7.2 R04.4.11
// `g[rs, cs]` on a grid is the sub-rectangle of rows `rs` and columns `cs`, walked row-major;
// an open range's `Int` end there is a position of its axis.
domain R = 0..3
domain C = 0..4
domain P = (R, C)
let ones: [R, C]Int = [P] p => 1
let g: [R, C]Int = [P] p => Int(p) + 1
print(ones[..2, ..2].sum(), ones[2.., 1..2].len)
let r = g[1..=2, 2..]
print(r.sum(), r.len)
var s = 0
for x in g[2.., ..2]: s = s * 100 + x
for x in g[..1, 3..]: s = s * 100 + x
print(s)
var n = 0
for (k, x) in g[1.., ..1].items: n += Int(k._0) - 1 + x - x
print(n + 1)
