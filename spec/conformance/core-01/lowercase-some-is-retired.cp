// expect: E0503 at 1
// rule: 01.3.1
// tool: check
// has: "`some` is spelled `Some`"
let a: ?Int = some(3)
print(a, none: -1)
