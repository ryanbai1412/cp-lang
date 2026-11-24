// expect: ok, output "3 2"
// rule: 05.5.2
// A fn that builds a struct from `List(xs)` owns the copy: `a` is free to change afterwards.
struct Box(v: List<Int>)
fn wrap(xs: List<Int>) -> Box = Box(List(xs))
var a = [1, 2]
let b = wrap(a)
a.push(3)
print(a.len, b.v.len)
