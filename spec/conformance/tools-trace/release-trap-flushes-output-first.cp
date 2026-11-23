// expect: exit 3
// rule: trace.1 trace.6
// tool: run --release
// input: "5"
// has: "before"
// The judge C++ rejects the input the checked run rejects, with the same exit status and the output written so far.
print("before")
let xs = [1, 2, 3]
let i: Int = read()
print(xs[i])
