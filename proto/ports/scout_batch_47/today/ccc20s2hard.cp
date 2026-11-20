// CCC '20 S2 (Hard) — Escape Room (Hard) (today form for proto/cpc)
// original: corpus/submissions/4324075.cpp
let n: 1..=4000 = read()
let m: 1..=4000 = read()

let target = n * m
domain Grid = 0..(n + 1) * (m + 1)
var arr: [Grid]Int = 0
for r in 0..n + 1:
    for c in 0..m + 1:
        let v: Int = read()
        arr[Grid(r * (m + 1) + c)] = v

// Flat 2-pass CSR adjacency (mirrors the memory-tight flat arena in 4324075.cpp).
domain Node = 0..target + 2
var deg: [Node]Int = 0
for r in 1..=n:
    for c in 1..=m:
        deg[Node(r * c)] += 1

var head: [Node]Int = 0
var cur: [Node]Int = 0
var off = 0
for u in 0..target + 1:
    head[Node(u)] = off
    cur[Node(u)] = off
    off += deg[Node(u)]
head[Node(target + 1)] = off

domain Edge = 0..target
var to: [Edge]Int = 0
for r in 1..=n:
    for c in 1..=m:
        let u = r * c
        let idx = cur[Node(u)]
        cur[Node(u)] = idx + 1
        to[Edge(idx)] = arr[Grid(r * (m + 1) + c)]

var vis: [Node]Bool = false
var stk: List<Int> = []
let start = arr[Grid((m + 1) + 1)]
if start >= 1 and start <= target:
    vis[Node(start)] = true
    stk.push(start)

var escaped = false
while stk.len > 0:
    let u = stk[stk.len - 1]
    stk.pop()
    if u == target:
        escaped = true
        break
    let lo = head[Node(u)]
    let hi = head[Node(u + 1)]
    for ei in lo..hi:
        let v = to[Edge(ei)]
        if v >= 1 and v <= target and not vis[Node(v)]:
            vis[Node(v)] = true
            stk.push(v)

if escaped: print("yes") else: print("no")
