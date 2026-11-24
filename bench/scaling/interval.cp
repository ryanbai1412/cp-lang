use std.interval.{IntervalMap, IntervalSet, SparseIntervalSet, SparseIntervalMap}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => v.ord % 2

for _ in 0..1 << 20:
    let m0 = IntervalMap<V, Int>(0) // timed: interval.IntervalMap.new
    s += m0.runs
for _ in 0..40:
    s += IntervalMap.from(a).runs // timed: interval.IntervalMap.from
var m = IntervalMap.from(a)
let even: Int -> Bool = x => x == 0 // within: interval.IntervalMap.first_where
for i in 0..reps:
    let p = i * 7919 % (n - 4)
    s += m.get(V(p)) // timed: interval.IntervalMap.get
    s += m.run_at(V(p)).val // timed: interval.IntervalMap.run_at
    m.assign(V(p)..V(p + 2), 1 - p % 2) // timed: interval.IntervalMap.assign [fixed: overwritten]
    m.assign(V(p)..V(p + 1), p % 2)
    m.assign(V(p + 1)..V(p + 2), 1 - p % 2)
    s += m.assign_point(V(p + 2), p % 2) // timed: interval.IntervalMap.assign_point
    m.set(V(p + 3), 1 - p % 2) // timed: interval.IntervalMap.set
    for q in m.range(V(p)..V(p + 3)): s += q.val // timed: interval.IntervalMap.range [fixed: k]
    s += (m.first_where(V.cuts.nth(p), even) ?? V(0)).ord // timed: interval.IntervalMap.first_where [fixed: skipped]
    s += m.runs // timed: interval.IntervalMap.runs
for _ in 0..40:
    for q in m.each(): // timed: interval.IntervalMap.each
        s += q.val // timed: interval.IntervalMap.each
    for q in m.each_rev(): // timed: interval.IntervalMap.each_rev
        s += q.val // timed: interval.IntervalMap.each_rev

for _ in 0..1 << 20:
    let e = IntervalSet<V>() // timed: interval.IntervalSet.new
    s += e.runs
var t = IntervalSet<V>()
for i in 0..n / 2: t.add(V(2 * i)..V(2 * i + 1))
for i in 0..reps:
    let p = i * 7919 % (n / 2 - 2) * 2
    t.add(V(p)..V(p + 2)) // timed: interval.IntervalSet.add [fixed: merged]
    t.remove(V(p + 1)..V(p + 2)) // timed: interval.IntervalSet.remove [fixed: split]
    if t.contains(V(p)): s += 1 // timed: interval.IntervalSet.contains
    if t.contains_range(V(p)..V(p + 1)): s += 1 // timed: interval.IntervalSet.contains_range
    s += (t.run_at(V(p)) ?? V(0)..V(0)).start.ord // timed: interval.IntervalSet.run_at
    s += (t.next_covered(V(p + 1)) ?? V(0)).ord // timed: interval.IntervalSet.next_covered
    s += (t.next_gap(V(p)) ?? V(0)).ord // timed: interval.IntervalSet.next_gap
    s += t.covered // timed: interval.IntervalSet.covered
    s += t.runs // timed: interval.IntervalSet.runs
for _ in 0..40:
    for r in t.each(): // timed: interval.IntervalSet.each
        s += r.start.ord // timed: interval.IntervalSet.each

for _ in 0..1 << 20:
    let e = SparseIntervalSet() // timed: interval.SparseIntervalSet.new
    s += e.runs
var u = SparseIntervalSet()
for i in 0..n: u.add(2 * i..2 * i + 1)
for i in 0..reps:
    let p = i * 7919 % (n - 2) * 2
    u.add(p..p + 2) // timed: interval.SparseIntervalSet.add [fixed: merged]
    u.remove(p + 1..p + 2) // timed: interval.SparseIntervalSet.remove [fixed: split]
    if u.contains(p): s += 1 // timed: interval.SparseIntervalSet.contains
    s += u.next_gap(p) // timed: interval.SparseIntervalSet.next_gap
    s += u.covered // timed: interval.SparseIntervalSet.covered
    s += u.runs // timed: interval.SparseIntervalSet.runs
for _ in 0..40:
    for r in u.each(): // timed: interval.SparseIntervalSet.each
        s += r.start // timed: interval.SparseIntervalSet.each

for _ in 0..1 << 20:
    let e = SparseIntervalMap(0, 1 << 40, 0) // timed: interval.SparseIntervalMap.new
    s += e.runs
print(s)
