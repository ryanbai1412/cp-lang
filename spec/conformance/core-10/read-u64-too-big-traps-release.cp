// expect: exit 3
// rule: 10.2.6
// tool: run --release
// input: "18446744073709551616\n"
// has: "`18446744073709551616` does not fit in a `U64`"
// The judge C++ traps on a number too big for its type with `cpl run`'s words, not a wrapped value.
let x = read<U64>()
print(x)
