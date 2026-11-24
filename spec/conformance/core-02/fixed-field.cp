// expect: E0405
// rule: 02.6.1
struct E: u: Int, w: Int
var e = E(1, 2)
e.w = 3 // <- here
