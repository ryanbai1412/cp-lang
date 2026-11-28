// expect: ok, output "5 3"
// rule: R02.10.2
// A range type's constant ends may use `**` and `<<`: they fold as the run-time operations do.
let z: -10 ** 9..=10 ** 9 = 5
const K = 4
let w: 0..1 << K = 3
print(z, w)
