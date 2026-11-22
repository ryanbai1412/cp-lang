// expect: E0904 at 3
// rule: 03.1.9
let n: 1..=1e9 = read()
domain V = 0..n
let a: [V]Int = [V].filled(0)
print(a[V(0)])
