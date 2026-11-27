// expect: R0203
// A `const` that traps when worked out is a compile error carrying the trap's code.
const Z = 0
const X = 10 / Z  // <- here
print(X)
