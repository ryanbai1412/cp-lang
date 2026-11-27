// expect: ok, output "2\n3 4\n12 1\n7 6 5\n6 6"
// rule: R04b.1.12
// A row `g[r]` of a grid is a view keyed by the column domain: its keys, cells and walks are
// `C`'s positions, not offsets from 0.
domain R = 0..2
domain C = 1..=3
let g: [R, C]Int = [R, C] (r, c) => 4 * Int(r) + Int(c)
let row = g[R(1)]
print(row.keys.find(c => row[c] == 6)!)
print(row.len, g[R(0)][C(3)] + 1)
print(row.sum() - 6, g[R(0)].nth(0))
print(List(x for x in row.rev).len + 4, row[C(3)] - 1, row.nth(0))
print(List(x for (c, x) in row.items if c == C(2))[0], g[R(1)][C(2)])
