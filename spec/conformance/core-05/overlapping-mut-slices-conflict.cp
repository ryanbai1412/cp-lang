// expect: E0601 at 7
// rule: 05.8.2
domain V = 1..=4
fn paint(mut x: [V]Int, mut y: [V]Int):
    for k in x.keys: x[k] = 1
    for k in y.keys: y[k] = 2
var a = [V] v => 0
let c = V(3).l
paint(mut a[..c], mut a[V(2).l..])
print(a[V(2)])
