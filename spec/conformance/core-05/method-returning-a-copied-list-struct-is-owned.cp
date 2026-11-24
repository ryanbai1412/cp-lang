// expect: ok, output "3 2"
// rule: 05.5.2
// A method that builds a struct from `List(xs)` owns the copy: `a` is free to change afterwards.
struct Box(v: List<Int>):
    fn with(self, xs: List<Int>) -> Box = Box(List(xs))
var a = [1, 2]
let b = Box([]).with(a)
a.push(3)
print(a.len, b.v.len)
