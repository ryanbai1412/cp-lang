let n: 1..=9 = read()
domain I = 1..=n
fn bump(mut a: [I]Int, i: I):
    a[i] += 1
var xs: [I]Int = 0
bump(xs, I.first)
