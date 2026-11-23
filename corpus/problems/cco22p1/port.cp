// Source: CCO '22 P1 (Alternating Heights), https://dmoj.ca/problem/cco22p1
// Problem: a line A_1..A_N of students 1..K (N <= 3000, K <= N, students may repeat) and
//   Q <= 1e6 queries x < y: can distinct heights give h[A_x] > h[A_x+1] < h[A_x+2] > ...
//   up to A_y? Print YES or NO for each.
// Solution: a range works iff the taller-than graph of its adjacent pairs is acyclic
//   (`consistent`: Kahn's algorithm, O(N + K)), and a range that works still works shrunk.
//   So binary-search, for each x, the furthest y that works (`reach[x]`): O(N^2 log N) in
//   all, then O(1) per query.
// original: corpus/submissions/4801881.cpp
let n: 2..=3000 = read()
let k: 2..=n = read()
let q: 1..=1e6 = read()
domain Spot = 1..=n
domain Student = 1..=k
let line: [Spot]Student = read()
let asks: [q](x: Spot, y: Spot) = read()
assert all(t.x < t.y for t in asks)

fn consistent(x: Spot, y: Spot) -> Bool:
    var taller: [Student]List<Student> = []
    var low = true
    for i in x..y:
        let j = Spot.at(i + 1)!
        if low: taller[line[i]].push(line[j])
        else: taller[line[j]].push(line[i])
        low = not low
    var waits: [Student]Int = 0
    for s in Student:
        for t in taller[s]: waits[t] += 1
    var free = [s for s in Student if waits[s] == 0]
    var placed = 0
    while let s = free.pop():
        placed += 1
        for t in taller[s]:
            waits[t] -= 1
            if waits[t] == 0: free.push(t)
    placed == k

let reach: [Spot]Spot = [Spot] x => do:
    var lo = x
    var hi = Spot.last!
    while lo < hi:
        let mid = Spot((lo.label + hi.label + 1) / 2)
        if consistent(x, mid): lo = mid
        else: hi = Spot.at(mid - 1)!
    lo
for (x, y) in asks: print(if y <= reach[x]: "YES" else: "NO")
