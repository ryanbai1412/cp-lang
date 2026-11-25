// expect: R0401
// rule: R10.1.1, diagnostics R0401
// input: "2\n1\n0\n"
// A trap in case 2 keeps case 1's output.
@!multitest
let x: Int = read()
assert x > 0, "positive"  // <- here
print(x)
