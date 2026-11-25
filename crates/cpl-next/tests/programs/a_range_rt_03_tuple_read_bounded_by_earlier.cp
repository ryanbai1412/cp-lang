// expect: R0502
// rule: R10.2.3
// input: "5 9\n"
// R10.2.3: a tuple read's later part may be bounded by an earlier one.
let (n, k): (1..=10, 1..=n) = read()  // <- here
print(n, k)
