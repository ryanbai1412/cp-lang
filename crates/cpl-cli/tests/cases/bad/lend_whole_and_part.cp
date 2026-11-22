// error: one contains the other
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
fn mix(mut xs: [V]Int, mut ys: [V]Int):
    for v in xs.positions: xs[v] += 1
mix(mut a, mut a[l..=r])
