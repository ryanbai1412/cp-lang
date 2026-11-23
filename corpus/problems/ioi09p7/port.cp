// IOI '09 P7 - Regions (25pt): n employees form a tree under employee 1, each
// from one of r home regions. Each query (r1, r2), answered online, asks for
// the number of pairs (a, b) with a from r1, b from r2 and a an ancestor of b.
// A region with more than sqrt(n) employees is "big": one sweep of the tree
// in preorder counts, for every other region at once, the pairs it tops.
// A small r1 walks its own members and counts the r2 members inside each
// subtree range by binary search over r2's preorder positions.
// original: corpus/submissions/5032754.cpp

let (n, r, q): (1..=2e5, 1..=25000, 1..=2e5) = read()
domain V = 1..=n
domain Region = 1..=r

var home: [V]Region = [V] _ => Region.first
var boss: [V]?V = none
let head: Region = read()
home[V.first] = head
for v in V:
    if v == V.first: continue
    let (b, h): (V, Region) = read()
    boss[v] = b
    home[v] = h
let t = Rooted.from_parents(boss)

var members: [Region]List<V> = [Region] _ => []
// spots[x]: preorder positions of the employees of region x, increasing
var spots: [Region]List<Int> = [Region] _ => []
for v in t.at:
    members[home[v]].push(v)
    spots[home[v]].push(t.pos(v).to_idx)

// how many employees of region x sit at preorder positions < p
// WISH TR4: `(0..w.len).first_true(i => w[i] >= p)` is on the cheat sheet
// but not in the interpreter prelude, and the real want is one call:
// `spots[x].count_in(t.sub(a))` on a sorted `List<Pre>`.
fn before(x: Region, p: Int) -> Int:
    let lo = (0..spots[x].len).first_true(mid => spots[x][mid] >= p) ?? spots[x].len
    lo

// pairs topped by the big region x, for every lower region at once
fn sweep(x: Region) -> [Region]Int:
    var above: [V]Int = 0
    var pairs: [Region]Int = 0
    for v in t.at:
        if let b = t.parent(v): above[v] = above[b]
        pairs[home[v]] += above[v]
        if home[v] == x: above[v] += 1
    pairs

var big: [Region]?[Region]Int = [Region] _ => none
var seen: Map<(Region, Region), Int> = Map()

for _ in 0..q:
    let (r1, r2): (Region, Region) = read()
    var ans = 0
    if members[r1].len * members[r1].len > n:
        big[r1] ??= sweep(r1)
        ans = big[r1]![r2]
    elif let hit = seen.get((r1, r2)):
        ans = hit
    else:
        for a in members[r1]:
            let s = t.sub(a)
            ans += before(r2, s.end.to_idx) - before(r2, s.start.to_idx)
        seen[(r1, r2)] = ans
    print(ans)
