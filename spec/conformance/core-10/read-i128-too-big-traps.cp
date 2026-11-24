// expect: R0502 (checked)
// rule: 10.2.6
// input: "170141183460469231731687303715884105728\n"
// has: "`170141183460469231731687303715884105728` does not fit in an `I128`"
// A whole number too big for its type traps naming it; read-i128-too-big-traps-release.cp is the C++ side.
let x = read<I128>() // <- here
print(x)
