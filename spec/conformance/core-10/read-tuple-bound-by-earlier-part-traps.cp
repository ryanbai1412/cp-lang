// expect: R0502 (checked)
// rule: 10.2.3
// input: "3 5\n"
// The later part's bound is checked against the earlier part's value.
let (n, k): (1..=10, 1..=n) = read() // <- here
print(n, k)
