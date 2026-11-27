// expect: E0209
// A `const` that reads itself has no value: an error, not a hang.
const A = A + 1  // <- here
print(A)
