// A `mut` arg beside a closure runs on the VM when the closure cannot reach
// the lent place: a by-ref capture of another local, and a call to a fn that
// reads only other globals.
var xs: List<Int> = [1, 2, 3]
var scale = 10
var log: List<Int> = []
fn weight(x: Int) -> Int = x * scale
fn reads_xs(x: Int) -> Int = x + xs.len
fn each_then_push(mut ys: List<Int>, f: (Int) -> ()):
    for y in ys:
        f(y)
    ys.push(ys.len)
for round in 0..2:
    var total = 0
    each_then_push(mut xs, x => total += weight(x) + round)
    print(xs, total)
    each_then_push(mut log, x => total += reads_xs(x))
    print(log, total)
