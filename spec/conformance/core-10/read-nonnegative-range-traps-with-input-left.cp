// expect: R0502 (checked)
// rule: 10.2.2
// input: "3\n1 10 2                                                                   \n"
// A value past a nonnegative range traps even with plenty of input after it.
let n: 1..=10 = read()
let a: [0..n]0..=9 = read() // <- here
print(a.sum())
