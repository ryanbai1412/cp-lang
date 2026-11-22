// error: s.end <= t.start or t.end <= s.start
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
fn mix(mut xs: [V]Int, mut ys: [V]Int):
    for v in xs.positions: xs[v] += 1
mix(mut a[l.l..r.r], mut a[r.l..r.r])
