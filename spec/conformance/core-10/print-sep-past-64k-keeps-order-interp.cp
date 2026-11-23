// expect: ok
// rule: 10.3.1
// tool: run
// has: "0\n0 0\n1 1\n"
// has: "\n6776 6776\n"
// Output printed before a big `print(xs, sep:)` stays before it in the interpreter (round 26, B2: the 64 KiB flush).
print(0)
print([(i, i) for i in 0..9000], sep: "\n")
