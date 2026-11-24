// Source: Yet Another Contest 1 P2 (A Boring Problem), https://dmoj.ca/problem/yac1p2
// Problem: a tree of 3 <= n <= 2e5 nodes, each black (B) or white (W). Count the simple paths
//   (unordered, single nodes included) that contain at least three nodes of the same colour.
// Solution: count the complement, paths with at most two nodes of each colour, and subtract
//   it from all n + n(n-1)/2 paths. down[u, b, w] counts the paths from u down into its
//   subtree with b black and w white nodes (both below 3). Children up: before merging child
//   c into u, pair c's paths with u's so far whose counts still sum below 3. O(81 n).
// original: corpus/submissions/4376253.cpp

let n: 3..=200_000 = read()
domain V = 1..=n
domain Few = 0..3
let colour: [V]Char = read()
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V(1))

var down: [V, Few, Few]Int = 0
var calm = 0
for u in t.order.rev:
    let mb = if colour[u] == 'B': 1 else: 0
    let mw = 1 - mb
    down[u, Few(mb), Few(mw)] = 1
    calm += 1
    for c in t.children(u):
        for b in Few:
            for w in Few:
                for b2 in Few:
                    for w2 in Few:
                        if Int(b) + Int(b2) < 3 and Int(w) + Int(w2) < 3: calm += down[u, b, w] * down[c, b2, w2]
        for b in Few:
            for w in Few:
                if let Some(b1) = Few.at(b + mb):
                    if let Some(w1) = Few.at(w + mw): down[u, b1, w1] += down[c, b, w]

print(n + choose2(n) - calm)
