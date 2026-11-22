// expect: ok, output "1 2"
// rule: 05.8.2 07.5.9
domain V = 1..=4
fn paint(mut x: [V]Int, mut y: [V]Int):
    x.fill(1)
    y.fill(2)
var a = [V] v => 0
let c = V(3).l
paint(mut a[..c], mut a[c..])
print(a[V(2)], a[V(3)])
