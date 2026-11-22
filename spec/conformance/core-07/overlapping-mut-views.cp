// expect: E0601 at 6
// rule: 07.5.9
domain V = 1..=4
fn paint(mut x: [V]Int, mut y: [V]Int):
    x.fill(1)
    y.fill(2)
var a = [V] v => 0
paint(mut a[..V(3).r], mut a[V(2).l..])
