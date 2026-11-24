// expect: R0101
// rule: trace.1
// tool: run
// input: "5"
// has: "before"
// has: "locals: xs = [1, 2, 3], i = 5"
// A trap in a checked run prints what was written before it, then the rule, the place and the locals in source terms.
print("before")
let xs = [1, 2, 3]
let i: Int = read()
print(xs[i]) // <- here
