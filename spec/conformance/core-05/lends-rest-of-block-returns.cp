// expect: ok, output "1"
// rule: 05.6.2
domain V = 1..=4
fn window(xs: [V]Int, l: V) lends [V]Int = xs[l..]
fn f(a: [V]Int) -> Int:
    let w = window(a, V(2))
    if w.sum() > 5: return 1
    0
print(f([V] v => Int(v)))
