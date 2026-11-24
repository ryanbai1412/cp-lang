// expect: E0503
// rule: 06.4.3
// tool: check
// has: "@check"
@checker // <- here
fn ok(n: Int) -> Bool = n > 0
let n: 1..=8 = read()
print(n)
