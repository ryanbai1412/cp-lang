// expect: E0503
// rule: 01.3.1
// tool: check
// has: "`some` is spelled `Some`"
let a: ?Int = some(3) // <- here
print(a, none: -1)
