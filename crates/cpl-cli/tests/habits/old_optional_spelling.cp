// expect: E0503
// rule: 01.3.1
// tool: check
// has: "`none` is spelled `None`"
let b: ?Int = none // <- here
print(b, none: -1)
