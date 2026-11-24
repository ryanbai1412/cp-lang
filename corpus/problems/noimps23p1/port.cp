// Source: NOI Provincial Selection 2023 P1, Train Station,
//   https://dmoj.ca/problem/noimps23p1
// limits: 1000 ms, 512 MB
// Problem: n stations on a line and m tracks [l, r] (l < r). A train leaves station x along any
//   track through x, in either direction, may change to another track at any station both
//   cover but never turns back, and stops at the end of a track. Print, ascending, every
//   station where it can stop. n, m <= 2e5.
// Solution: rightward stops are the right ends of tracks reachable from x, where a track is
//   reachable if it covers some position in [x, r] of a reachable track. A DFS over tracks
//   sweeps each position once with path-compressed skip pointers `nxt`, adding the tracks that
//   start there. Leftward stops are the same on the mirrored line. O((n + m) log n).
// original: corpus/submissions/5556659.cpp
let (n, m, start1): (Int, Int, Int) = read()
var ivs: List<(l: Int, r: Int)> = []
for _ in 0..m:
    let (l, r): (Int, Int) = read()
    ivs.push((l: l - 1, r: r - 1))

// Right endpoints reachable from `start` by riding intervals rightwards.
fn reach(start: Int, iv: List<(l: Int, r: Int)>) -> List<Bool>:
    var starts_at: List<List<Int>> = [[] for _ in 0..n]
    for i in 0..m: starts_at[iv[i].l].push(i)
    // nxt[x]: smallest unswept position >= x (path-compressed skip pointers).
    var nxt: List<Int> = [x for x in 0..n + 1]
    var seen = List.filled(m, false)
    var hit = List.filled(n, false)
    var stack: List<Int> = []
    for i in 0..m:
        if iv[i].l <= start < iv[i].r:
            seen[i] = true
            stack.push(i)
    while stack.len > 0:
        let i = stack.pop()
        hit[iv[i].r] = true
        var x = max(start, iv[i].l)
        while true:
            var root = x
            while nxt[root] != root: root = nxt[root]
            while nxt[x] != root:
                let y = nxt[x]
                nxt[x] = root
                x = y
            x = root
            if x > iv[i].r: break
            nxt[x] = x + 1
            for j in starts_at[x]:
                if not seen[j]:
                    seen[j] = true
                    stack.push(j)
    hit[start] = false
    return hit

let start = start1 - 1
let right = reach(start, ivs)
var mirrored: List<(l: Int, r: Int)> = [(l: n - 1 - iv.r, r: n - 1 - iv.l) for iv in ivs]
let left = reach(n - 1 - start, mirrored)
for x in 0..n:
    if right[x] or left[n - 1 - x]: print(x + 1)
