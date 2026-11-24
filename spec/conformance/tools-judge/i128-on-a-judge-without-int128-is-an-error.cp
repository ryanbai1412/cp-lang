// expect: exit 1
// rule: judge.3
// tool: build --judge cses -o -
// file: cp.toml "[judge.cses]\nint128 = false\n"
// has: "judge `cses`"
// has: "__int128"
// A project that says its judge's compiler has no `__int128` gets an error naming that judge for a program using `I128`, not C++ that fails on submission.
let a: I128 = read()
print(a * a)
