// expect: E0701
// rule: 08.1.4
// tool: check
// has: "`List.sum` is the type's own member"
// has: "rename the fn, or call it as a plain fn: `sum([1, 2])`"
// Members are found before extension fns, so an extension a member always wins over is never called: an error, not dead code.
fn sum(self: List<Int>) -> Int = 7
print([1, 2].sum()) // <- here
