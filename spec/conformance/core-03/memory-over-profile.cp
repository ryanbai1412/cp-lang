// expect: E0904
// gap: compiler 2026-11-24T05Z
// rule: 03.1.9
let n: 1..=1e9 = read()
domain V = 0..n
let a: [V]Int = [V].filled(0) // <- here
print(a[V(0)])
