// expect: R0001
// rule: 12.14.3
// input: "3\n"
// An intrinsic traps where its C++ form fails: `sqrt` of a negative number.
let n: Int = read()
print(sqrt(n - 5)) // <- here
