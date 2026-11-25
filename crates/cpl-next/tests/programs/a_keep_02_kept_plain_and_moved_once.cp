// expect: ok, output "3 2"
// A kept plain argument copies; an owned heap one moves in once and is not used after.
fn put<T>(mut into: List<T>, x: T):
    into.push(x)
var ns: List<Int> = []
let k = 4
for i in 0..3:
    put(mut ns, k)
var ls: List<List<Int>> = []
put(mut ls, [1])
let ys = [2, 3]
put(mut ls, ys)
print("{ns.len} {ls.len}")
