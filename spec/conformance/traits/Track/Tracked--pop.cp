// expect: ok, output "4"
fn evict<S: Track<Int>>(mut s: S, x: Int) = s.pop(x)
var t = track(0, (mut s, x: Int) => s += x, (mut s, x: Int) => s -= x)
t.push(3)
t.push(4)
evict(mut t, 3)
print(t.state)
