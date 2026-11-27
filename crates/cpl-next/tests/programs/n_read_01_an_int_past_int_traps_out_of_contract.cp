// expect: R0502
// rule: R10.2.6
// input: "3 -9223372036854775808\n"
// A whole number past `Int`, the lowest bit pattern included, is out of the input's contract, not a bad token.
let a: Int = read()
let b: Int = read() // <- here
print(a, b)
