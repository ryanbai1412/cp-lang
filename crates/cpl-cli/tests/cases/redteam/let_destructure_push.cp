// `let (a, b) = xs[i]` owns its two values: the pushes below move the list's storage, and under ASan a
// binding that still pointed into the old storage is a heap-use-after-free.
var xs: List<(Int, Int)> = [(1, 2), (3, 4)]
for i in 0..2:
    let (a, b) = xs[i]
    for k in 0..100: xs.push((a + k, b))
    print(a, b)
fn nested(n: Int) -> Int:
    var ys: List<(Int, List<Int>)> = [(1, [2]), (3, [4])]
    let (a, b) = ys[1]
    for k in 0..n: ys.push((7, [7]))
    a + b[0] + ys.len
print(nested(100), xs.len)
// the same for the subject of a `match`, and for the value an `if let` unwraps
enum Op:
    Add(x: Int, y: Int)
    Skip
var ops: List<Op> = [Add(1, 2), Skip]
match ops[0]:
    Add(x, y) =>
        for k in 0..100: ops.push(Skip)
        print(x, y, ops.len)
    Skip =>
        print(0)
var zs: List<?(Int, Int)> = [(1, 2), none]
if let (a, b) = zs[0]:
    for k in 0..100: zs.push(none)
    print(a, b, zs.len)
// and a later write to the table does not change what the `let` bound
var tab: [0..3](Int, Int) = (1, 2)
let (ta, tb) = tab[0]
tab[0] = (5, 6)
print(ta, tb)
