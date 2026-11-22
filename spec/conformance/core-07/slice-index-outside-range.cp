// expect: R0101 at 4 (checked)
// rule: 07.4.6
domain V = 0..5
let a: [V]Int = [V] v => Int(v)
let s = a[V(1)..V(3)]
print(s[V(4)])
