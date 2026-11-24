// expect: R0202
// rule: 12.14.3
// input: "126\n"
// `abs` of `I128.MIN` traps on both engines: no `I128` holds its absolute value.
let n: Int = read()
let m: I128 = (I128(1) << n) * -2
print(abs(m)) // <- here
