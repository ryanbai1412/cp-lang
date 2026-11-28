// Source: Codeforces 2150E2 (Hidden Single (Version 2)),
//   https://codeforces.com/contest/2150/problem/E2
// limits: 5000 ms, 250 MB
// Problem: interactive, multitest (t <= 20), n = 300. A hidden array of length 2n - 1 holds
//   every value 1..n twice except one, which appears once. A query "? x |S| S..." answers
//   whether value x occurs at some position in S. Print "! y", the single value, using at
//   most 925 queries per test.
// Solution: shuffle the positions with a fixed seed, then cut them in two halves. Asking
//   each value in the left half, and in the right half when it is on the left, gives its
//   left count 0, 1 or 2 (about 2n queries). The half whose counts sum to more than its
//   length holds the single; only values seen twice there are candidates. Bisect each
//   candidate: a sub-range without it is dropped, and a value present in both sub-halves
//   is a split pair, not the single, so most candidates die in a query or two.
// Bounds: n: 1..=300

let wire = Wire()
use std.rand.Rng

const SHUFFLE_SEED = 48
var rng = Rng.new(SHUFFLE_SEED)

@!multitest
let n: 1..=300 = read()
domain Val = 1..=n
domain Pos = 0..2 * n - 1

var order = List(1..2 * n)
rng.shuffle(mut order)
let perm = [Pos] p => order[p.ord]

fn has(v: Val, lo: Pos.bounds, hi: Pos.bounds) -> Bool:
    if lo == hi: return false
    print("?", v, hi.ord - lo.ord, perm[lo..hi])
    read<Int>() == 1

let mid = Pos.cuts.nth((2 * n - 1) / 2)
var left_w: [Val]0..=2 = 0
for v in Val:
    left_w[v] = if not has(v, Pos.bounds.first!, mid):
        0
    elif has(v, mid, Pos.bounds.last!):
        1
    else:
        2

let (lo, hi, only_here) = if left_w.sum() > mid.ord:
    (Pos.bounds.first!, mid, [Val] v => left_w[v] == 2)
else:
    (mid, Pos.bounds.last!, [Val] v => 2 - left_w[v] == 2)

fn is_single(v: Val, start: Pos.bounds, stop: Pos.bounds) -> Bool:
    var (l, r) = (start, stop)
    while r.ord - l.ord > 1:
        let m = Pos.cuts.nth((l.ord + r.ord) / 2)
        if not has(v, l, m): l = m
        elif not has(v, m, r): r = m
        else: return false
    true

for v in Val if only_here[v]:
    if is_single(v, lo, hi):
        print("!", v)
        break
