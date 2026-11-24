// expect: ok, output "7"
fn add<S: Track<Int>>(mut s: S, x: Int) = s.push(x)
var t = track(0, (mut s, x: Int) => s += x, (mut s, x: Int) => s -= x)
add(mut t, 3)
add(mut t, 4)
print(t.state)
