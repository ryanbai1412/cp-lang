// error: would keep the lent slice
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
fn top(xs: [V]Int) -> [V]Int:
    xs[l..=r]
print(top(a))
