// expect: ok, output "3 1"
fn evict<S: Track<Int>>(mut s: S, x: Int) = s.pop(x)
var w = Window<Int, (Int, Int) -> Int>.new((a: Int, b: Int) => a + b)
w.push(2)
w.push(3)
evict(mut w, 2)
print(w.all!, w.len)
