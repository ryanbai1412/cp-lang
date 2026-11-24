// Source: MMCC '14 P3 (Esdeath), https://dmoj.ca/problem/mmcc14p3
// Problem: a tree of n <= 5000 cities; put s <= n soldiers on cities so that every city is
//   within D roads of a soldier. Print the minimum D.
// Solution: the soldiers needed fall as D grows, so bisect D. For a fixed D, go from the
//   leaves up: each vertex keeps its farthest uncovered descendant and its nearest soldier
//   below; a soldier goes on a child whose uncovered vertex would be D + 1 away, and one on
//   the root if anything is left uncovered. O(n log n).
// original: corpus/submissions/5120083.cpp

let (n, k): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let t = Rooted(u => g.out(u), V.first!)

fn outposts(radius: Int) -> Int:
    var open: [V]?Int = [V] _ => None  // farthest uncovered vertex below, if any
    var post: [V]?Int = [V] _ => None  // nearest outpost below, if any
    var used = 0
    for v in t.at.rev:
        var far = 0
        var near: ?Int = None
        for c in t.children(v):
            if let d = post[c]: near = min(near ?? n, d + 1)
            if let d = open[c]:
                if d + 1 > radius:
                    used += 1
                    near = 1
                else:
                    far max= d + 1
        post[v] = near
        open[v] = if far + (near ?? n) <= radius: None else: far
    if open[V.first!] != None: used += 1
    used

// WISH TR4: `(0..n).first_true(r => outposts(r) <= k)`
let lo = (0..n).first_true(mid => outposts(mid) <= k) ?? n
print(lo)
