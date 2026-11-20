// CCO 2022 P3: two disjoint schedules of intervals on sides 0 and 1, with a
// travel delay k to switch sides. Maximise the number of intervals visited.
// Run Dijkstra over DP states (count, side, other_active) -> earliest time d,
// transitioning to the next interval on the same side or switching after d + k
// via binary search on sorted interval starts.
// original: corpus/submissions/4801968.cpp
let n: 1..=300000 = read()
let m: 1..=300000 = read()
let k: 1..=1000000000 = read()

struct Seg:
    l: Int
    r: Int

let raw0: [n]Seg = read()
let raw1: [m]Seg = read()
let a0 = sorted(raw0, by: s => (s.l, s.r))
let a1 = sorted(raw1, by: s => (s.l, s.r))

// Last interval index with l <= t, or none if every interval starts after t.
fn last_starting_le(xs: List<Seg>, t: Int) -> ?Int:
    var lo = 0
    var hi = xs.len
    while lo < hi:
        let mid = (lo + hi) / 2
        if xs[mid].l <= t: lo = mid + 1
        else: hi = mid
    if lo == 0: none else: lo - 1

let total = n + m
var dist: [0..=total, 0..2, 0..2]?Int = none
var pq = MinHeap<(Int, Int, Int, Int)>()
var ans = 0

fn push_state(u: Int, loc: Int, other: Int, d: Int):
    ans max= u
    let prev = dist[u, loc, other]
    if prev == none or d < prev!:
        dist[u, loc, other] = d
        pq.push((d, u, loc, other))

push_state(1, 0, 0, a0[0].l)
for s in a1:
    if k < s.r:
        push_state(1, 1, 0, max(s.l, k))

while let top = pq.pop():
    let d = top.0
    let u = top.1
    let loc = top.2
    let other = top.3
    if dist[u, loc, other] != d: continue
    let mine = if loc == 0: a0 else: a1
    let theirs = if loc == 0: a1 else: a0
    let mi = last_starting_le(mine, d)!
    let oi = last_starting_le(theirs, d)

    if mi + 1 < mine.len:
        let t = mine[mi + 1].l
        var keep = 0
        if other == 1:
            if let o = oi:
                if t < theirs[o].r: keep = 1
        push_state(u + 1, loc, keep, t)

    var cand = (last_starting_le(theirs, d + k) ?? -1)
    if cand < 0: cand = 0
    if other == 1:
        if let o = oi:
            if o == cand: cand += 1
    if cand < theirs.len:
        if d + k >= theirs[cand].r: cand += 1
        if cand < theirs.len:
            let t = max(d + k, theirs[cand].l)
            let keep = Int(t < mine[mi].r)
            push_state(u + 1, 1 - loc, keep, t)

print(ans)
