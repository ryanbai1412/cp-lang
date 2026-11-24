// n: 4096
// A map of n runs is a tree of n nodes. n stays small so it still fits in L2 at 4n: past it
// the first read of a random coordinate pays a cache miss the cost model does not count, and
// `get` would measure the memory hierarchy, not the fn.
use std.interval.{SparseIntervalMap}
let n: Int = read()
let reps = 1 << 17
var s = 0
var w = SparseIntervalMap(0, 1 << 40, 0)
for i in 0..n: w.assign(2 * i..2 * i + 1, 1)
let bump: Int -> Int = x => 1 - x // within: interval.SparseIntervalMap.update
for i in 0..reps:
    let p = i * 7919 % (n - 2) * 2
    s += w.get(p) // timed: interval.SparseIntervalMap.get
    s += w.run_at(p).val // timed: interval.SparseIntervalMap.run_at
    w.assign(p..p + 2, 0) // timed: interval.SparseIntervalMap.assign [fixed: removed]
    w.assign(p..p + 1, 1)
    w.update(p + 1..p + 2, bump) // timed: interval.SparseIntervalMap.update [fixed: touched]
    w.update(p + 1..p + 2, bump)
    for q in w.range(p..p + 3): s += q.val // timed: interval.SparseIntervalMap.range [fixed: k]
    s += w.runs // timed: interval.SparseIntervalMap.runs
for _ in 0..160:
    for q in w.each(): // timed: interval.SparseIntervalMap.each
        s += q.val // timed: interval.SparseIntervalMap.each
    for q in w.each_rev(): // timed: interval.SparseIntervalMap.each_rev
        s += q.val // timed: interval.SparseIntervalMap.each_rev
print(s)
