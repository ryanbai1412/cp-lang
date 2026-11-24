// Source: Codeforces 2084F (Skyscape), https://codeforces.com/contest/2084/problem/F
// limits: 4000 ms, 500 MB
// Problem: a permutation a and a partial permutation c (0 = missing) of length n. b is good if
//   at most n operations "pick l < r with a_r = min(a_l..a_r), rotate a_l..a_r right by one" turn
//   a into b. Print a good b that extends c, or -1. n <= 5e5, sum of n <= 5e5; multitest.
// Solution: a smaller value can only move left past larger ones, so u < v with u before v in a
//   keep that order in b. v must sit in [lo[v], hi[v]]: lo is the largest fixed position of a
//   u <= v at or before v in a, hi the smallest of a u >= v at or after v (sweeps with a max
//   Fenwick over value order, hi on reversed values and positions); a fixed v needs both at its
//   own cell. Earliest deadline first fills the free positions in order from a MinHeap of (hi, v) over the free values, failing on a
//   missed deadline; ties go to the smaller value, which keeps the order. O(n log n).
// Bounds: a: 1..=Pos.count

@!multitest
use std.seq.{buckets}

const N = 500_000
domain Pos = read(2..=N)
domain Val = 1..=Pos.count
let a: [Pos]Val = read()
let c: [Pos]?Val = read(none: 0)
let fixed: [Val]?Pos = c.inverse()

fn reach<S: Seq>(order: S, flip: Bool) -> [Val]0..N:
    let n = Pos.count
    var t: [0..n]0..N = 0
    var out: [Val]0..N = 0
    for v in order:
        let k = if flip: n - 1 - v.ord else: v.ord
        if let p = fixed[v]:
            let s = if flip: n - 1 - p.ord else: p.ord
            var i = k
            while i < n:
                t[i] max= s
                i |= i + 1
        var r = 0
        var i = k + 1
        while i > 0:
            r max= t[i - 1]
            i &= i - 1
        out[v] = r
    out

let lo: [Val]Pos = reach(a, flip: false).map(r => Pos.nth(r))
let hi: [Val]Pos = reach(a.rev, flip: true).map(r => Pos.nth(Pos.count - 1 - r))

let by_lo = buckets(lo)
var pq = MinHeap<(Pos, Val)>()
var b: [Pos]Val = Val.first!

for i in Pos:
    for v in by_lo[i]:
        if fixed[v] == None: pq.push((hi[v], v))
    if let want = c[i]:
        if lo[want] != i or hi[want] != i: return print(-1)
        b[i] = want
        continue
    let (deadline, v) = pq.pop() ?? return print(-1)
    if deadline < i: return print(-1)
    b[i] = v

print(b)
