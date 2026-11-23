// expect: E0503 at 1
// rule: 01.3.1
// tool: check
// has: "`none` is spelled `None`"
let b: ?Int = none
print(b, none: -1)
