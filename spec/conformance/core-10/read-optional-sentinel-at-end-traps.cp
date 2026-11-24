// expect: R0501 (checked)
// rule: 10.2.10
// input: "4"
// With `none: -1` the `-1` is the `None`; the end of input is not, so the second read traps.
let a = read<?Int>(none: -1)
let b = read<?Int>(none: -1) // <- here
print(a ?? 0, b ?? 0)
