// expect: R0202
// rule: 01.5.8
// `2 ** 63` is past 64 bits and traps like `*`.
let k = 63
print(2 ** k) // <- here
