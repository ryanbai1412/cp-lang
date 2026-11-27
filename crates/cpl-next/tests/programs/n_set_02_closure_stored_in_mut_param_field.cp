// expect: ok, output "42 9 3"
// rule: R05.10.2
// A fn that stores a closure in a field of a struct it is lent `mut`, or pushes one onto a list
// it is lent `mut`, adds it to the set of that field or list in the caller.
struct Box:
    var f: () -> Int
fn arm(mut b: Box, n: Int):
    b.f = () => n + 1
fn grow(mut fs: List<Int -> Int>) -> Int:
    fs.push(x => x * 3)
    fs.len
var b = Box(() => 0)
arm(mut b, 41)
var fs: List<Int -> Int> = []
let n = grow(mut fs)
let g = fs[0]
print(b.f(), g(3), n + 2)
