// expect: ok
// rule: trace.2
// tool: run --trace
// has: ".trace.html"
// `--trace` writes one HTML file beside the program and says where, and the program's own output is unchanged.
let xs = [1, 2, 3]
print(xs[1])
