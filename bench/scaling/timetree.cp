// n: 2048
use std.timetree.{TimeTree, TimeStep, cdq, offline_over}
let n: Int = read()
let reps = 1 << 18
var cnt = 0
let mark: () -> Int = () => cnt // within: timetree.TimeTree.run
let apply: Int -> () = e => cnt += e // within: timetree.TimeTree.run
let answer: Int -> Int = t => cnt // within: timetree.TimeTree.run
let undo: Int -> () = m => cnt = m // within: timetree.TimeTree.run
let omark: () -> Int = () => cnt // within: timetree.offline_over
let oapply: Int -> () = e => cnt += e // within: timetree.offline_over
let oanswer: Int -> Int = t => cnt // within: timetree.offline_over
let oundo: Int -> () = m => cnt = m // within: timetree.offline_over
let by: Int -> (Int, Int) = e => ((e * 7919) % n, (e * 7919) % n + 1 + (e * 31) % (n / 2)) // within: timetree.offline_over
let cross: (List<Int>, List<Int>) -> () = (l, r) => cnt += l.len // within: timetree.cdq
var s = 0
for _ in 0..16000:
    let t = TimeTree<Int>(n) // timed: timetree.TimeTree.new
    s += t.steps().len
var tt = TimeTree<Int>(n)
for i in 0..n:
    let lo = (i * 7919) % n
    tt.add_interval(lo, min(n, lo + 1 + (i * 31) % (n / 2)), i % 5) // timed: timetree.TimeTree.add_interval
    tt.add_range(lo..min(n, lo + 1 + (i * 17) % (n / 2)), i % 3) // timed: timetree.TimeTree.add_range
var t2 = TimeTree<Int>(n)
for i in 0..16 * n:
    let lo = (i * 7919) % n
    t2.add_interval(lo, min(n, lo + 1 + (i * 31) % (n / 2)), i % 5) // timed: timetree.TimeTree.add_interval
    t2.add_range(lo..min(n, lo + 1 + (i * 17) % (n / 2)), i % 3) // timed: timetree.TimeTree.add_range
for i in 0..reps:
    s += tt.event((i * 13) % (2 * n)) // timed: timetree.TimeTree.event
for _ in 0..24:
    s += tt.steps().len // timed: timetree.TimeTree.steps
    s += tt.run(mark:, apply:, answer:, undo:).len // timed: timetree.TimeTree.run
    s += offline_over(n, [i for i in 0..n], by:, mark: omark, apply: oapply, answer: oanswer, undo: oundo).len // timed: timetree.offline_over
    cdq([i for i in 0..n], on_cross: cross) // timed: timetree.cdq
for _ in 0..3:
    for st in tt.walk(): // timed: timetree.TimeTree.walk
        s += 1 // timed: timetree.TimeTree.walk
print(s, cnt)
