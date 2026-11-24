// Source: IOI 2009 Day 2 (Regions), https://dmoj.ca/problem/ioi09p7
// limits: 7000 ms, 128 MB
// Problem: a tree of N <= 2e5 employees rooted at 1 (each one's supervisor is more senior),
//   each from one of R <= 25000 regions. Answer Q <= 2e5 queries (r1, r2) online: the number
//   of pairs (e1, e2) with e1 from r1, e2 from r2 and e1 a proper ancestor of e2.
// Solution: lay the tree out in preorder and list each region's preorder positions. A
//   region with more than sqrt(N) members is big: one sweep, counting each node's ancestors
//   in it, answers it against every r2 at once and is cached. A small r1 walks its members
//   and counts r2's positions inside each subtree range (a SortedSet range), memoized per
//   (r1, r2): O(N sqrt N + Q sqrt N log N).
// original: corpus/submissions/5032754.cpp
// Bounds: n, q: 1..=2e5; r: 1..=25000; head: 1..=r; b: 1..=n; h, r1, r2: 1..=r


let (n, r, q): (1..=2e5, 1..=25000, 1..=2e5) = read()
domain V = 1..=n
domain Region = 1..=r

let head: Region = read()
var home: [V]Region = [V] _ => head
var boss: [V]?V = None
for v in V.all.drop(1):
    let (b, h): (V, Region) = read()
    boss[v] = Some(b)
    home[v] = h
let t = Rooted.from_parents(boss)

var members: [Region]List<V> = [Region] _ => []
var spots: [Region]SortedSet<t.Pre> = [Region] _ => SortedSet()
for v in V:
    members[home[v]].push(v)
    spots[home[v]].add(t.pos(v))

// pairs topped by the big region x, for every lower region at once
fn sweep(x: Region) -> [Region]Int:
    var above: [V]Int = 0
    var pairs: [Region]Int = 0
    for v in t.at:
        if let b = t.parent(v): above[v] = above[b]
        pairs[home[v]] += above[v]
        if home[v] == x: above[v] += 1
    pairs

var big: [Region]?[Region]Int = [Region] _ => None
var small: Map<(Region, Region), Int> = Map()
for _ in 0..q:
    let (r1, r2): (Region, Region) = read()
    if members[r1].len * members[r1].len > n:
        big[r1] ??= sweep(r1)
        print(big[r1]![r2])
    else:
        small[(r1, r2)] ??= sum(spots[r2].range(t.sub(a)).len for a in members[r1])
        print(small[(r1, r2)])
