// expect: ok, output "0 -5"
// rule: 02.8.3
// input: "-5 0 5\n"
// `[3]-5..=5` is an array of the range `-5..=5`; the leading minus belongs to the bound (rebuild 7, H-NEG-RANGE-TYPE).
let a: [3]-5..=5 = read()
print(a.sum(), a.min()!)
