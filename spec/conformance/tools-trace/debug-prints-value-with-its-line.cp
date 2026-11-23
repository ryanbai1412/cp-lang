// expect: ok, output "4"
// rule: trace.4
// tool: run
// has: "[line "
// has: "x = 3"
// `debug(x)` prints the expression and its value with its line, apart from the program's output; `build` drops it (bundle.2).
let x = 3
debug(x)
print(x + 1)
