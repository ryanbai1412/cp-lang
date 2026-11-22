use std.interval.{IntervalMap}
use std.rand.{Rng}

domain P = 1..=12
var m = IntervalMap<P, Int>.new(0)
var dense: [P]Int = 0

fn check_sync():
    for i in P:
        assert m[i] == dense[i]
    var prev_val = -999
    var covered = 0
    for (r, v) in m.each():
        assert v != prev_val
        prev_val = v
        covered += r.len
    assert covered == P.count

// Single-run split both sides, same-value no-op, left/right coalesce, default no-op on_old
var acc = 0
fn add_acc(r: Range<P>, v: Int): acc += r.len * (v + 1)
m.assign(P(4)..=P(8), 5, on_old: add_acc)
for i in P(4)..=P(8): dense[i] = 5
check_sync()
m.assign(P(5)..=P(7), 5, on_old: add_acc)
check_sync()
// Single-run sl < lo with right coalesce (hi == seg.hi and next run has same val)
m.assign(P(6)..=P(8), 0)
for i in P(6)..=P(8): dense[i] = 0
check_sync()
// Single-run sl == lo with both left and right coalesce
m.assign(P(4)..=P(5), 0)
for i in P(4)..=P(5): dense[i] = 0
check_sync()
assert m.runs == 1

// assign_point
let prev = m.assign_point(P(6), 7)
dense[P(6)] = 7
check_sync()
assert prev == 0

// Randomized brute-checked stress across all single-run & multi-run coalescing cases
var rng = Rng(20261121)
var total_overwritten = 0
for _ in 0..150:
    let a = rng.int(1, 12)
    let b = rng.int(a, 12)
    let val = rng.int(0, 3)
    var expected = 0
    for i in a..=b:
        expected += dense[P(i)]
        dense[P(i)] = val
    var got = 0
    fn add_got(r: Range<P>, v: Int): got += r.len * v
    m.assign(P(a)..=P(b), val, on_old: add_got)
    assert got == expected
    total_overwritten += got
    check_sync()

print(acc, m.runs, total_overwritten)
