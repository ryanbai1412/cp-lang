// expect: E0302
// rule: 07.9.3
let n: 1..=5 = read()
domain V = 1..=n
var a: [V]Int = read()
let f = V.first!
let k: Int = a[f..f].max() // <- here
print(k + 1)
// Seq.max() on a slice view returns ?T, not T (must not assign to Int without unwrap).
