// expect: E0303 at 5
// rule: 04.2.12
domain V = 0..4
var d: [Pair<V>]Int = 0
d[0, 1, 2] = 7
