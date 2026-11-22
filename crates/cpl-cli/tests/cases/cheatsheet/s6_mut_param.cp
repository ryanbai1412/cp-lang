let n: 1..=10 = read()
domain V = 1..=n
var a: [V]Int = read()
let v = V(2)
fn swap_adj(mut a: [V]Int, i: V):
    if let j = i + 1: swap(mut a[i], mut a[j])
swap_adj(mut a, V(1))
print(a)
