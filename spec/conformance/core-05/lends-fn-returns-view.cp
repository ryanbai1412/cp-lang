// expect: ok, output "7"
// rule: 05.6.1 05.7.1 05.7.8 07.5.10
domain V = 1..=4
fn window(xs: [V]Int, l: V) lends [V]Int = xs[l..]
let a = [V] v => Int(v)
print(window(a, V(3)).sum())
