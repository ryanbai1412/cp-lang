// expect: exit 3
// rule: 10.2.6
// tool: run --release
// input: "99999999999999999999\n"
// has: "`99999999999999999999` does not fit in an `Int`"
// The judge C++ traps on a number too big for its type with `cpl run`'s words, not a wrapped value.
let x = read<Int>()
print(x)
