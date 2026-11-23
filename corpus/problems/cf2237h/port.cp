// Source: Codeforces 2237H (Slime and Queries), https://codeforces.com/contest/2237/problem/H
// Problem: a tree with n <= 1e5 vertices and a slime on m connected vertices; a Move frees one
//   occupied vertex and occupies a new one, staying connected. Food appears at c_1..c_q in order
//   (forced online: c_i is decoded with ans_{i-1}); for each i print the fewest Moves to eat
//   c_1..c_i in order. Multitest, sums of n and q <= 1e5.
// Solution: walk the food path cur -> c and stamp it with the query time in an IntervalMap over
//   HLD preorder. Slime cells sit in buckets keyed by when they last moved; a path vertex last
//   stamped at t_old is free if a cell that last moved at or before t_old is matched to it, else
//   it costs a Move (`missing`, the answer). Chunks in decreasing t_old take from the newest
//   eligible bucket (older cells serve more vertices, so this is optimal by exchange) and matched
//   cells move to bucket t_now: about O((n + q) log^2 n).

@!multitest
let (n, m, q): (2..=100_000, 2..=100_000, 1..=100_000) = read()
domain V = 1..=n
domain Slime = 0..m
domain Time = 1..=q + 2
let never = Time.first
let start = Time.first!.offset(1)!
let tree = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), V.first!)
let init: [Slime]V = read()

// Partition of preorder positions by last-painted timestamp.
var paint = IntervalMap<tree.Pre, Time>.new(never!)
for v in init: paint.assign(tree.pos(v)..=tree.pos(v), start)

// Slime waiting to move, keyed by the time it was last moved; only non-empty buckets.
var supply = Map<Time, Int>()
supply[start] = m

// Matches up to `need` units from the newest buckets at or before `t`.
// Each emptied bucket leaves `supply`, so all calls cost O((q + m) log q) total.
fn take(need: Int, t: Time) -> Int:
    var got = 0
    while got < need, let (u, cnt) = supply.last_le(t):
        let k = min(cnt, need - got)
        got += k
        if k == cnt: supply.remove(u) else: supply[u] -= k
    got

var cur = init[Slime.last!]
var missing = 0
var ans: List<Int> = []

for t_now in Time.first!.offset(2)!..=Time.last!:
    let c = V.wrap(Int(read<V>()) + missing)
    var pieces: List<(t: Time, need: Int)> = []
    fn record(r: Range<tree.Pre>, t: Time):
        pieces.push((t:, need: r.len))
    for (range, _) in tree.chains(cur, c):
        paint.assign(range, t_now, on_old: record)
    let chunks = pieces.sorted(by: x => desc(x.t))
    cur = c

    let demand = chunks.map(ch => ch.need).sum()
    let moved = chunks.map(ch => take(ch.need, ch.t)).sum()
    missing += demand - moved
    if moved > 0: supply[t_now] = moved
    ans.push(missing)

print(ans)
