// Tuned: both tries are flat child lists walked in place, as the accepted C++'s node pools are,
// where the port keeps a `Trie` object per column behind a `HashMap` and builds each walk's path
// and colour terms as lists.
//
// Source: CCO 2017 Day 1 P3 (Vera and Modern Art), https://dmoj.ca/problem/cco17p3
// limits: 4000 ms, 1024 MB
// Problem: N <= 2e5 strokes x y v (x, y <= 1e18, v <= 1e4): with a, b the largest powers of
//   two not above x and y, add v to every point (x + a p, y + b q), p, q >= 0. Then answer
//   Q <= 2e5 queries: the total colour at point (r, c), r, c <= 1e18.
// Solution: write x as a leading 1 and k low bits; a stroke reaches row r exactly when r
//   has the same k low bits and more than k bits. So insert each x's low bits, lowest
//   first, into a binary trie, each x node owning a binary trie over y holding the colour
//   sums. A query walks r's bits down the x trie and, at every node on that path with a
//   column, walks c's bits down the column's y trie adding the colour sums: O(60) per stroke, O(60^2) per query.
// Bounds: n, q: 1..=2e5; strokes.x, strokes.y, queries.r, queries.c: 1..=1e18; strokes.v: 1..=1e4
// The bits below x's leading one, lowest first: they name x's residue class.
// The trie nodes that x's low bits walk through, root first, as far as the trie goes.

type Coord = 1..=1e18

let (n, q): (1..=2e5, 1..=2e5) = read()
let strokes: [n](x: Coord, y: Coord, v: 1..=1e4) = read()
let queries: [q](r: Coord, c: Coord) = read()

// Both tries live in flat lists: node o's child on bit b is kid[2 * o + b], 0 when absent. x node 0
// is the root; y node 0 is a sentinel, so column[o] == 0 says x node o has no column yet.
// Each trie has at most 1 + 59 n nodes, and a colour sum is at most n * 1e4.
type Node = 0..=12e6
var xkid: List<Node> = [0, 0]
var column: List<Node> = [0]  // the root of x node's y trie
var ykid: List<Node> = [0, 0]
var colour: List<0..=2e9> = [0]  // the colour sum at each y node
for s in strokes:
    var o = 0
    for i in 0..bit_width(s.x) - 1:
        let at = 2 * o + (s.x >> i & 1)
        if xkid[at] == 0:
            xkid[at] = column.len
            column.push(0)
            xkid.push(0)
            xkid.push(0)
        o = xkid[at]
    if column[o] == 0:
        column[o] = colour.len
        colour.push(0)
        ykid.push(0)
        ykid.push(0)
    var p = column[o]
    for i in 0..bit_width(s.y) - 1:
        let at = 2 * p + (s.y >> i & 1)
        if ykid[at] == 0:
            ykid[at] = colour.len
            colour.push(0)
            ykid.push(0)
            ykid.push(0)
        p = ykid[at]
    colour[p] += s.v

for qu in queries:
    let (kr, kc) = (bit_width(qu.r) - 1, bit_width(qu.c) - 1)
    var total = 0
    var o = 0
    var i = 0
    while o != 0 or i == 0:
        if column[o] != 0:
            var p = column[o]
            var j = 0
            while p != 0:
                total += colour[p]
                if j == kc: break
                p = ykid[2 * p + (qu.c >> j & 1)]
                j += 1
        if i == kr: break
        o = xkid[2 * o + (qu.r >> i & 1)]
        i += 1
    print(total)
