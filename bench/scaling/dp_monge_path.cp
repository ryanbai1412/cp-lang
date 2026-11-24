use std.dp.{MongePath}
let n: Int = read()
let reps = 1 << 17
var s = 0
let xs = [i * 7919 % n for i in 0..n]

let sq_path: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.monge_path
let sq_next: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.MongePath.next
for _ in 0..8:
    var p = MongePath(xs.sorted(), sq_next)
    for _ in 0..n - 1:
        s += p.next().prev // timed: dp.MongePath.next
var mp = MongePath(xs.sorted(), sq_path)
for _ in 0..n - 1: s += mp.next().prev
for i in 0..reps:
    s += mp.val(i % n) // timed: dp.MongePath.val
    s += mp.len // timed: dp.MongePath.len
    if mp.is_empty: s += 1 // timed: dp.MongePath.is_empty
for _ in 0..200:
    s += mp.items.count() // timed: dp.MongePath.items
print(s)
