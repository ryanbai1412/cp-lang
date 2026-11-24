// Disjoint `mut` slices passed to one fn each write their own cells,
// whether their root is a local, a parameter or a top-level array.
domain V = 0..6
fn paint(mut x: [V]Int, mut y: [V]Int, c: Int):
    for k in x.keys: x[k] = c
    for k in y.keys: y[k] = c + 1
fn local(c: Int) -> [V]Int:
    var a = [V] v => 0
    let m = V(3).l
    for i in 0..2:
        paint(mut a[..m], mut a[m..], c + i)
    a
var t = [V] v => 0
let m = V(2).l
paint(mut t[..m], mut t[m..], 5)
print(local(1), t)
