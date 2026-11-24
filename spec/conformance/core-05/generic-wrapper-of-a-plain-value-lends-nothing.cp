// expect: ok, output "1 3"
// rule: 05.5.3
// A plain argument for the type parameter is copied in, and the source is free after the result's last use.
struct Box<S>(src: S)
fn wrap<S>(s: S) -> Box<S> = Box(s)
var n = 1
let b = wrap(n)
n += 1
var xs = [1, 2]
let c = wrap(xs)
let k = c.src.len
xs.push(9)
print("{b.src} {xs.len}")
