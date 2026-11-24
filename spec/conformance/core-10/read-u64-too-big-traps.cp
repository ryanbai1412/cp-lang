// expect: R0502 (checked)
// rule: 10.2.6
// input: "18446744073709551616\n"
// has: "`18446744073709551616` does not fit in a `U64`"
// A whole number too big for its type traps naming it; read-u64-too-big-traps-release.cp is the C++ side.
let x = read<U64>() // <- here
print(x)
