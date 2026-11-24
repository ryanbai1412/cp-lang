// n: 1024
// The sweep and point-set algorithms of std.geometry. Points are scattered by multiplying their
// index by large primes.
use std.geometry.*
let n: Int = read()
let pts = [Pt(i * 7919 % 100003, i * 104729 % 100019) for i in 0..n]
let vs = [Vec(F64(p.x), F64(p.y)) for p in pts]
let segs = [(a: Pt(i * 37 % 1000, i), b: Pt(i * 37 % 1000 + 500, i)) for i in 0..n]
let rects = [Rect(i * 7919 % 100003..i * 7919 % 100003 + 5000, i * 104729 % 100019..i * 104729 % 100019 + 5000) for i in 0..n]
let circles = [Circle(Vec(F64(i * 37 % 1000), F64(i * 91 % 1000)), 50.0) for i in 0..n / 8]
var acc = 0
var f = 0.0
for r in 0..8:
    acc += (any_segments_meet(segs) ?? (0, 0)).0 // timed: geometry.any_segments_meet
    acc += (any_segments_meet_idx(segs) ?? (0, 0)).0 // timed: geometry.any_segments_meet_idx
    acc += rects_union_area(rects) % 7 // timed: geometry.rects_union_area
for r in 0..32:
    acc += closest_pair2(pts) % 7 // timed: geometry.closest_pair2
    acc += closest_pair(pts).dist2 % 7 // timed: geometry.closest_pair
for r in 0..64:
    f += circles_union_area(circles) // timed: geometry.circles_union_area
for r in 0..128:
    f += min_enclosing_circle(vs).r // timed: geometry.min_enclosing_circle
print(acc, yes(f > 0.0))
