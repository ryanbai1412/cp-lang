// expect: ok, output "3 2"
// rule: 10.2.3
// input: "3 2\n"
// A tuple read's later part may be bounded by an earlier one (acceptance run 4, tuple-read-dependent-bound).
let (n, k): (1..=10, 1..=n) = read()
print(n, k)
