// expect: ok, output "28"
// rule: 10.2.2
// input: "6\n+5 0000000000000000000007 -0 3 9 4                                        \n"
// A range starting at 0 or above reads a sign, leading zeros and `-0` as any `Int` does, however much input follows.
let n: 1..=10 = read()
let a: [0..n]0..=9 = read()
print(a.sum())
