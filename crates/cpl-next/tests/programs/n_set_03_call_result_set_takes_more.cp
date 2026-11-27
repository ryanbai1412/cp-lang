// expect: ok, output "4 10 2 3\n8 7"
// rule: R05.10.2
// The fns a call returns are a set the call passes: where the caller keeps the result, it takes
// more fns, and the callee's instance for that call builds its own closures into it.
struct Mon:
    var op: (Int, Int) -> Int
fn mk(k: Int) -> List<Int -> Int> = [x => x + k]
fn mon(k: Int) -> Mon = Mon((a, b) => a + b + k)
var fs = mk(3)
fs.push(x => x * 5)
let g: Int -> Int = x => x - 1
fs.push(g)
var m = mon(5)
let before = m.op(1, 2)
m.op = (a, b) => a * b + 1
print(fs[0](1), fs[1](2), fs[2](3), fs.len)
print(before, m.op(2, 3))
