// expect: E0304
// gap: compiler 2026-11-24T05Z
// rule: 04b.1.12
// `g.rows` is an O(1) view, so it is a getter with no `()`, like `.len` and `.rev`.
domain Row = 0..2
domain Col = 0..3
let g: [Row, Col]Int = [Row, Col] (r, c) => Int(r) * 3 + Int(c)
print(g.rows().len) // <- here
