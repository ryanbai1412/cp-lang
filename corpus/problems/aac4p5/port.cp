// Source: An Animal Contest 4 P5 (Neighbourly Neighbourhoods), https://dmoj.ca/problem/aac4p5
// Problem: n, m, q <= 5e5, 1 <= x <= n. Print m distinct directed edges, no self loops, whose
//   graph has exactly x SCCs and puts each of the q given pairs in one SCC; else -1.
// Solution: unite the pairs; -1 if fewer than x groups. Merging two groups never shrinks the
//   room for edges, so keep the x - 1 smallest groups and merge the rest. A group of size
//   s > 1 needs an s-cycle; the other ordered pairs inside a group and the pairs from an
//   earlier group to a later one add no SCC merges. Possible iff the cycles fit in m and
//   m fits in the room; list the first m of those edges. O(n log n + m).
// original: corpus/submissions/4140767.cpp
let (n, m, x, q): (1..=5e5, 0..=5e5, 1..=5e5, 0..=5e5) = read()
domain V = 1..=n
var dsu = Dsu<V>()
for (u, v) in read<[q](V, V)>():
    dsu.unite(u, v)

// Merging always grows the room, so keep the x - 1 smallest groups and merge the rest.
let groups = dsu.groups().sorted(by: g => g.len)
fn components() -> List<List<V>>:
    var kept = groups.take(x - 1).collect()
    kept.push(groups.drop(x - 1).flatten().collect())
    kept

// Cycles first (each group of two or more must be strongly connected), then the other
// ordered pairs inside a group, then pairs from an earlier group to a later one.
fn edges(comps: List<List<V>>) -> yields (V, V):
    for c in comps if c.len > 1:
        for (i, v) in c.items:
            yield (v, c[(i + 1) % c.len])
    for c in comps:
        for (i, u) in c.items:
            for (j, v) in c.items if j != i and j != (i + 1) % c.len:
                yield (u, v)
    for (i, a) in comps.items:
        for b in comps.drop(i + 1):
            for u in a:
                for v in b:
                    yield (u, v)

if groups.len < x:
    print(-1)
else:
    let comps = components()
    let forced = sum(c.len for c in comps if c.len > 1)
    let room = sum(c.len * (c.len - 1) for c in comps) + (n * n - sum(c.len * c.len for c in comps)) / 2
    if forced <= m <= room:
        for (u, v) in edges(comps).take(m):
            print(u, v)
    else:
        print(-1)
