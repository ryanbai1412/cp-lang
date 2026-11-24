// expect: ok, output "9 50 cde\n2 2"
// rule: 07.4.17
domain V = 0..5
fn total<K>(w: [K]Int) -> Int = sum(w)
let xs = [1, 2, 3, 4, 5]
let a: [V]Int = [10, 20, 30, 40, 50]
let s = "abcdef"
let w = xs[1..4]
let d = a[V(1)..V(3)]
let t = s[2..5]
print(total(w), total(d), t)
print(w.first!, d.len + t.len - 3)
