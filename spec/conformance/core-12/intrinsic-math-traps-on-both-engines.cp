// expect: R0401 (checked)
// rule: 12.14.3
// input: "3\n"
// A library precondition around an intrinsic traps on both engines: std's `sqrt` asserts
// `x >= 0` before its `@intrinsic` row, which itself never traps.
let n: Int = read()
print(sqrt(n - 5)) // <- here
