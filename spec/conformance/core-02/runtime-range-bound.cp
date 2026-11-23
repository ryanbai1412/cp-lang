// expect: ok, output "3"
// rule: 02.9.1
// input: "5\n3\n"
// A range type's bound may be a runtime value read earlier (rebuild 7, H-RUNTIME-RANGE-BOUND).
let n: 1..=10 = read()
let k: 1..=n = read()
print(k)
