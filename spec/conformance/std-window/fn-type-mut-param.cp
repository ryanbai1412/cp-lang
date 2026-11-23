// expect: ok, output "12 10"
// rule: 06.1.2
fn apply(mut x: Int, y: Int, f: (mut Int, Int) -> ()):
    f(mut x, y)
var a = 7
apply(mut a, 5, (mut s, d) => s += d)
var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x)
t.push(4)
t.push(6)
print(a, t.state)
