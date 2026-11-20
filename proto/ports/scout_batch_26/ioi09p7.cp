// IOI '09 P7 - Regions (25pt): n employees form a tree under employee 1, each
// from one of r home regions. Each query (r1, r2), answered online, asks for
// the number of pairs (a, b) with a from r1, b from r2 and a an ancestor of b.
// A region with more than sqrt(n) employees is "big": one sweep of the tree
// in preorder counts, for every other region at once, the pairs it tops.
// A small r1 walks its own members and counts the r2 members inside each
// subtree range by binary search over r2's preorder positions.
// original: corpus/submissions/5032754.cpp
use std.tree.{Rooted}

let (n, r, q): (1..=2e5, 1..=25000, 1..=2e5) = read()
domain V = 1..=n
domain Region = 1..=r

var home: [V]Region = [V] _ => Region.first
var kids: [V]List<(to: V)> = [V] _ => []
let head: Region = read()
home[V.first] = head
for v in V:
    if v == V.first: continue
    // WISH TR3: `let boss: [V]?V = read()` with "the root has no entry" said
    // in the type, then `Rooted<V>.from_parents(boss)`; today the child lists
    // are built by hand only to be handed to `Rooted.new` as an adjacency.
    let (boss, h): (V, Region) = read()
    kids[boss].push((to: v))
    home[v] = h
let t = Rooted<V>.new(u => kids[u], V.first)

var members: [Region]List<V> = [Region] _ => []
// spots[x]: preorder positions of the employees of region x, increasing
var spots: [Region]List<Int> = [Region] _ => []
for p in Rooted<V>.Pre:
    let v = t.at[p]
    members[home[v]].push(v)
    spots[home[v]].push(p.to_idx)

// how many employees of region x sit at preorder positions < p
// WISH TR4: `bsearch_first(0..w.len, i => w[i] >= p)` is on the cheat sheet
// but not in the interpreter prelude, and the real want is one call:
// `spots[x].count_in(t.sub(a))` on a sorted `List<Pre>`.
fn before(x: Region, p: Int) -> Int:
    var (lo, hi) = (0, spots[x].len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if spots[x][mid] < p: lo = mid + 1
        else: hi = mid
    lo

// pairs topped by the big region x, for every lower region at once
fn sweep(x: Region) -> [Region]Int:
    var above: [V]Int = 0
    var pairs: [Region]Int = 0
    for p in Rooted<V>.Pre:
        let v = t.at[p]
        if let b = t.up[v]: above[v] = above[b]
        pairs[home[v]] += above[v]
        if home[v] == x: above[v] += 1
    pairs

var big: [Region]?[Region]Int = [Region] _ => none
var seen: Map<(Region, Region), Int> = Map()

for _ in 0..q:
    let (r1, r2): (Region, Region) = read()
    var ans = 0
    if members[r1].len * members[r1].len > n:
        if big[r1] == none: big[r1] = sweep(r1)
        ans = big[r1]![r2]
    elif let hit = seen.get((r1, r2)):
        ans = hit
    else:
        for a in members[r1]:
            let s = t.sub(a)
            ans += before(r2, s.hi.to_idx) - before(r2, s.lo.to_idx)
        seen[(r1, r2)] = ans
    print(ans)
