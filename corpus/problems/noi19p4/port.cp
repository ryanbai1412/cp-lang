// Source: NOI 2019 P4 (Jump), https://dmoj.ca/problem/noi19p4
// Problem: n <= 7e4 cities at distinct points of a w x h grid (w, h <= n) and m <= 1.5e5 portals;
//   portal i at city p_i takes time t_i <= 1e4 to any city in [L_i, R_i] x [D_i, U_i]. Print the
//   least time from city 1 to each of cities 2..n.
// Solution: Dijkstra over cities and portals. A portal's distance is final when popped, so every
//   city in its rectangle not yet reached gets exactly that distance and can be removed. A static
//   segment tree over x keeps, per node, its cities sorted by y with path-compressed next-alive
//   links; a rectangle harvests its O(log w) nodes from the first y >= D, skipping dead cities,
//   and each city is unlinked from each of its nodes at most once.
//   O((n log w + m log w log n) + (n + m) log(n + m)).
// original: corpus/submissions/3800719.cpp
// Rethought without std::set (which is missing in proto/cpc and tight on
// memory).
let n: 1..=70000 = read()
let m: 1..=150000 = read()
let w: 1..=70000 = read()
let h: 1..=70000 = read()

struct City:
    x: Int
    y: Int
    id: Int

var cities: [0..n]City = City(x: 0, y: 0, id: 0)
for i in 0..n:
    let x: Int = read()
    let y: Int = read()
    cities[i] = City(x:, y:, id: i)

struct Portal:
    lx: Int
    rx: Int
    ly: Int
    ry: Int

var adj: [0..n]List<(Int, Int)> = []
var portals: [0..m]Portal = Portal(lx: 0, rx: 0, ly: 0, ry: 0)
for i in 0..m:
    let u: 1..=n = read()
    let t: Int = read()
    let lx: Int = read()
    let rx: Int = read()
    let ly: Int = read()
    let ry: Int = read()
    adj[u - 1].push((n + i, t))
    portals[i] = Portal(lx:, rx:, ly:, ry:)

// Build static X-segment tree of y-sorted buckets by inserting cities in
// increasing y order (zero per-node sorting needed!).
let tree_sz = 4 * w + 4
var bucket_y: [0..tree_sz]List<Int> = []
var bucket_id: [0..tree_sz]List<Int> = []
var bucket_nxt: [0..tree_sz]List<Int> = []

fn seg_insert(o: Int, l: Int, r: Int, x: Int, y: Int, id: Int):
    let idx = bucket_y[o].len
    bucket_y[o].push(y)
    bucket_id[o].push(id)
    bucket_nxt[o].push(idx)
    if l == r: return
    let mid = (l + r) / 2
    if x <= mid: seg_insert(o * 2, l, mid, x, y, id)
    else: seg_insert(o * 2 + 1, mid + 1, r, x, y, id)

let by_y = cities.sorted(by: c => (c.y, c.id))
for c in by_y:
    seg_insert(1, 1, w, c.x, c.y, c.id)

// Sentinel slot at the end of each bucket for path-compressed next-alive.
for o in 0..tree_sz:
    let len = bucket_nxt[o].len
    if len > 0:
        bucket_nxt[o].push(len)

fn find_alive(o: Int, i0: Int) -> Int:
    var i = i0
    while bucket_nxt[o][i] != i:
        i = bucket_nxt[o][i]
    var cur = i0
    while cur != i:
        let nx = bucket_nxt[o][cur]
        bucket_nxt[o][cur] = i
        cur = nx
    i

fn lower_y(o: Int, target: Int) -> Int:
    let lo = (0..bucket_y[o].len).first_true(mid => bucket_y[o][mid] >= target) ?? bucket_y[o].len
    lo

var rem: [0..n]Bool = false
var popped: List<Int> = []

fn seg_harvest(o: Int, l: Int, r: Int, qlx: Int, qrx: Int, qly: Int, qry: Int):
    if bucket_y[o].len == 0: return
    if qlx <= l and r <= qrx:
        let len = bucket_y[o].len
        var idx = find_alive(o, lower_y(o, qly))
        while idx < len and bucket_y[o][idx] <= qry:
            let cid = bucket_id[o][idx]
            if not rem[cid]:
                rem[cid] = true
                popped.push(cid)
            bucket_nxt[o][idx] = find_alive(o, idx + 1)
            idx = bucket_nxt[o][idx]
        return
    let mid = (l + r) / 2
    if qlx <= mid: seg_harvest(o * 2, l, mid, qlx, qrx, qly, qry)
    if mid < qrx: seg_harvest(o * 2 + 1, mid + 1, r, qlx, qrx, qly, qry)

var dist: [0..n + m]?Int = None
var pq = MinHeap<(Int, Int)>()

fn push_dist(u: Int, d: Int):
    let prev = dist[u]
    if prev == None or d < prev!:
        dist[u] = d
        pq.push((d, u))

push_dist(0, 0)
while let top = pq.pop():
    let d = top.0
    let u = top.1
    if dist[u] != d: continue
    if u < n:
        for e in adj[u]:
            push_dist(e.0, d + e.1)
    else:
        let p = portals[u - n]
        popped = []
        seg_harvest(1, 1, w, p.lx, p.rx, p.ly, p.ry)
        for cid in popped:
            push_dist(cid, d)

for i in 1..n:
    print(dist[i]!)
