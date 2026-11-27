// expect: R0501
// input: "  \n"
// rule: R10.2.6
// A `Str` read with no token left traps as a bad token, like an `Int` read.
let s: Str = read() // <- here
print(s)
