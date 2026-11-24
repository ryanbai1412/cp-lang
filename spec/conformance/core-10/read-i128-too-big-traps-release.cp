// expect: exit 3
// rule: 10.2.6
// tool: run --release
// input: "170141183460469231731687303715884105728\n"
// has: "`170141183460469231731687303715884105728` does not fit in an `I128`"
// The judge C++ traps on a number too big for its type with `cpl run`'s words, not a wrapped value.
let x = read<I128>()
print(x)
