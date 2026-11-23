// Source: CCO 2022 P3 Double Attendance, https://dmoj.ca/problem/cco22p3
// Problem: two rooms show slides during open intervals (a, b): n1, n2 <= 3e5, no overlap within a
//   room, times <= 1e9. You start in room 1 at time 0; moving between rooms takes k <= 1e9. A
//   slide counts if you spend positive time in its room while it shows; print the most slides.
// Solution: Dijkstra over states (slides seen, room, whether the slide showing in the other room
//   is already counted), each keeping its earliest time d. From a state, wait for the next slide
//   in this room, or move and catch the first uncounted slide still showing at d + k (binary
//   search on sorted starts). 4 (n1 + n2) states, O((n1 + n2) log) in all.
// original: corpus/submissions/4801968.cpp
let n: 1..=300000 = read()
let m: 1..=300000 = read()
let k: 1..=1000000000 = read()

struct Seg:
    l: Int
    r: Int

let raw0: [n]Seg = read()
let raw1: [m]Seg = read()
let a0 = raw0.sorted(by: s => (s.l, s.r))
let a1 = raw1.sorted(by: s => (s.l, s.r))

// Last interval index with l <= t, or none if every interval starts after t.
fn last_starting_le(xs: List<Seg>, t: Int) -> ?Int:
    let lo = xs.first_gt(t, by: v => v.l) ?? xs.len
    if lo == 0: None else: lo - 1

let total = n + m
var dist: [0..=total, 0..2, 0..2]?Int = None
var pq = MinHeap<(Int, Int, Int, Int)>()
var ans = 0

fn push_state(u: Int, loc: Int, other: Int, d: Int):
    ans max= u
    let prev = dist[u, loc, other]
    if prev == None or d < prev!:
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
