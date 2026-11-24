// expect: R0502 (checked)
// rule: 10.2.6
// input: "99999999999999999999\n"
// has: "`99999999999999999999` does not fit in an `Int`"
// A whole number too big for its type traps naming it; read-int-too-big-traps-release.cp is the C++ side.
let x = read<Int>() // <- here
print(x)
