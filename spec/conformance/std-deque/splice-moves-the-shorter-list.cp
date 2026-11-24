// expect: ok, output "ok"
// rule: std.deque.LinkedList.splice_back
// `splice_back`/`splice_front` keep order whichever list is longer, also after a `reverse` on
// either side, and leave the spliced-into list usable at both ends; checked against Lists.
use std.deque.{LinkedList}

var ok = true
for round in 0..300:
    let na = rand(0, 6)
    let nb = rand(0, 6)
    var xs = [rand(0, 99) for _ in 0..na]
    var ys = [rand(0, 99) for _ in 0..nb]
    var a = LinkedList.from_list(xs)
    var b = LinkedList.from_list(ys)
    if rand(0, 1) == 1:
        a.reverse()
        xs.reverse()
    if rand(0, 1) == 1:
        b.reverse()
        ys.reverse()
    var want: List<Int> = []
    if rand(0, 1) == 0:
        a.splice_back(b)
        want = xs + ys
    else:
        a.splice_front(b)
        want = ys + xs
    a.push(100)
    a.push_front(-1)
    want = [-1] + want + [100]
    if a.each().collect() != want or a.each_rev().collect() != want.reversed() or a.len != want.len: ok = false
print(if ok: "ok" else: "mismatch")
