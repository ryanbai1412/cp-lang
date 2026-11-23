// expect: E0301 at 5
// rule: 04.2.11
let (h, w) = read<(2..=1000, 2..=1000)>()
domain Row = 1..=h
domain Col = 1..=w
var ways: [Row, Col]Int = [Row, Col] (r, c) => 0
ways[(Row.first, Col.first)] = 1
