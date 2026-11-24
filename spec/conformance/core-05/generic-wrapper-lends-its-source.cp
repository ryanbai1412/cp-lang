// expect: E0602
// rule: 05.5.3
// A user struct holding a type-parameter field lends what it was built from, with no `lends`.
struct Box<S>(src: S)
fn wrap<S>(s: S) -> Box<S> = Box(s)
var xs = [1, 2, 3]
let b = wrap(xs)
xs.push(9) // <- here
print(b.src.len)
