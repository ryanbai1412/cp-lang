// expect: ok, output "long 5"
// rule: trace.4
// tool: run
// has: "[line 9] x = 1"
// has: "[line 12] n = 2"
// A `debug(...)` that is a value, a closure's body or an `if` branch,
// prints as one that stands alone does.
let xs = [1, 2, 3, 4, 5]
let note = (x: Int) => debug(x)
note(xs[0])
let report = (n: Int) =>
    if n > 3: print("long", n) else: debug(n)
report(xs.len)
report(2)
