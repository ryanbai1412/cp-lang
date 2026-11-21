use std.rand.Rng
use std.seq.{
    next_permutation,
    nth_element,
    prev_permutation,
    reverse,
    rotate_left,
    rotate_right,
    scan_in_place,
    suffix_scan_in_place,
}

// 1. List<Int> mutable slice views: sort, reverse, rotate, scan, perm, nth_element, fill, shuffle, index write.
var xs = [9, 4, 1, 5, 2, 8]
xs[1..5].sort()
print(xs)

xs[1..5].reverse()
print(xs)

xs[1..5].rotate_left(1)
print(xs)

xs[1..5].rotate_right(1)
print(xs)

var ys = [10, 1, 2, 3, 4, 20]
ys[1..5].scan_in_place((a, b) => a + b)
print(ys)

var zs = [10, 3, 1, 4, 2, 20]
zs[1..5].suffix_scan_in_place((a, b) => max(a, b))
print(zs)

var ps = [9, 1, 2, 3, 9]
let ok1 = next_permutation(mut ps[1..4])
print(ok1, ps)
let ok2 = prev_permutation(mut ps[1..4])
print(ok2, ps)

var ns = [99, 5, 1, 4, 2, 3, 99]
nth_element(mut ns[1..6], 2)
print(ns[0], ns[3], ns[6])

var fs = [1, 2, 3, 4, 5]
fs[1..4].fill(7)
print(fs)

fs[1..4][1] = 42
print(fs)
fs[1..4].swap(0, 1)
print(fs)

var ss = [10, 1, 2, 3, 4, 20]
var rng = Rng(42)
rng.shuffle(mut ss[1..5])
print(ss[0], ss[5], ss[1..5].sorted())

// Let-bound, nested, and disjoint mutable slices.
var ds = [6, 5, 4, 3, 2, 1]
let v = ds[1..5]
v[1..3].sort()
print(ds)

let left = ds[0..3]
let right = ds[3..6]
left.sort()
right.reverse()
print(ds)

// 2. Fixed array [D]Int (1-based nominal domain).
domain D = 1..=6
var a: [D]Int = [D] i => 7 - i.ord
a[D(2)..D(6)].sort()
print(a)
a[D(2)..=D(5)].reverse()
print(a)
a[D(2)..D(6)].rotate_left(1)
print(a)
a[D(2)..D(5)].scan_in_place((x, y) => x + y)
print(a)
a[D(2)..D(6)].swap(0, 3)
print(a)

var p_arr: [D]Int = [D] i => i.ord
let ok3 = next_permutation(mut p_arr[D(2)..D(5)])
print(ok3, p_arr)

var sh_arr: [D]Int = [D] i => i.ord
var rng2 = Rng(7)
rng2.shuffle(mut sh_arr[D(2)..D(6)])
print(sh_arr[D(1)], sh_arr[D(6)], sh_arr[D(2)..D(6)].sorted())

// 3. Cut-domain array [D.bounds]Int.
var cuts: [D.bounds]Int = [D.bounds] c => 10 - c.ord
let c1 = D(2).l
let c2 = D(5).r
cuts[c1..c2].sort()
print(cuts)
cuts[c1..c2].reverse()
print(cuts)
cuts[c1..c2].rotate_right(1)
print(cuts)
cuts[c1..c2].scan_in_place((x, y) => min(x, y))
print(cuts)
cuts[c1..c2].set_at(0, 8)
cuts[c1..c2].swap(0, 3)
print(cuts)

// 4. Range-indexed fixed array [0..6]Int.
var ra: [0..6]Int = 0
for i in 0..6: ra[i] = 6 - i
ra[1..5].sort()
print(ra)
ra[1..5].reverse()
print(ra)
