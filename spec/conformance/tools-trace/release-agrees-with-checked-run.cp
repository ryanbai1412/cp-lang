// expect: ok, output "before\n3"
// rule: trace.6
// tool: run --release
// input: "2"
// On input it accepts, the judge C++ prints what the interpreter prints.
print("before")
let xs = [1, 2, 3]
let i: Int = read()
print(xs[i])
