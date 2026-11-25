// expect: ok, output "1\n2\n3"
// rule: R01.5 chained comparisons: each operand evaluated once
// input: "3\n1\n2\n3\n"
// H6: a chained comparison evaluates its middle operand once (R01 grammar: a < b < c).
var calls = 0
fn mid() -> Int:
    calls += 1
    5
@!multitest
let x: Int = read()
if 0 < mid() < 10: print(calls)
