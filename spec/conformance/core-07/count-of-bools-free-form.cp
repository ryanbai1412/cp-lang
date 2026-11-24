// expect: E0303
// rule: 07.9.4
// has: "len"
// The free form `count(b)` is the same question as `b.count()`; a generator of tests, `count(x for x in b)`, counts the trues (decision 53).
let b = [true, false, true]
print(count(x for x in b))
print(count(b)) // <- here
