// expect: ok, output "2 3\n3 12\n3 4 5\n12 4\n1 7"
// rule: R04b.1.12 R11.3.4
// `g.rows` and `g.cols` are indexed views keyed by the grid's domains: `g.rows[r]` is `g[r]`,
// `g.cols[c]` is `g[.., c]`, and their methods need no `use`, wherever the type came from.
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
print(g.rows.len, g.cols.len)
print([row.sum() for row in g.rows])
print([col.max()! for col in g.cols])
print(g.rows[R(1)].sum(), g.cols[C(1)][R(1)])
print(g.rows.keys.len - 1, g.cols.nth(2).sum())
