// expect: R0102
// rule: diagnostics R0102
// input: "3\n"
// D(x) out of the domain.
let n: Int = read()
domain V = 0..n
let v = V(n)  // <- here
print(Int(v))
