// expect: E0302
// rule: 04.2.29
// has: "not `P`"
struct P(a: Int)
let p = P(3)
domain D = p // <- here
print(1)
