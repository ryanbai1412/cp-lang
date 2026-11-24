// expect: ok, output "3 2"
// rule: 05.5.2
// A static-fn constructor lends its parameters only for the call: `a` is free to change afterwards.
struct Box(v: List<Int>):
    fn of(xs: List<Int>) -> Box = Box(List(xs))
var a = [1, 2]
let b = Box.of(a)
a.push(3)
print(a.len, b.v.len)
