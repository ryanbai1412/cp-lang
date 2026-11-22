// expect: E0503 at 1
// rule: 06.4.3
// tool: check
// has: "@check"
@checker
fn ok(n: Int) -> Bool = n > 0
let n: 1..=8 = read()
print(n)
