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
use std.strings.{Trie}

type Coord = 1..=1e18
domain Bit = 0..2

// The bits below x's leading one, lowest first: they name x's residue class.
fn low_bits(x: Coord) -> List<Bit> = [Bit(x >> i & 1) for i in 0..bit_width(x) - 1]

// The trie nodes that x's low bits walk through, root first, as far as the trie goes.
fn walk(t: Trie<Bit>, x: Coord) -> List<Int>:
    var path = [t.root]
    for i in 0..bit_width(x) - 1:
        if let next = t.step(path.last!, Bit(x >> i & 1)): path.push(next)
        else: break
    path

struct Column:
    ys: Trie<Bit>
    colour: List<Int>  // the colour sum at each ys node

let (n, q): (1..=2e5, 1..=2e5) = read()
let strokes: [n](x: Coord, y: Coord, v: 1..=1e4) = read()
let queries: [q](r: Coord, c: Coord) = read()

var xs = Trie<Bit>.new()
var column_at = HashMap<Int, Int>()  // x trie node -> index into cols
var cols: List<Column> = []
for s in strokes:
    let node = xs.add(low_bits(s.x))
    if node not in column_at:
        column_at[node] = cols.len
        cols.push(Column(ys: Trie<Bit>.new(), colour: []))
    let k = column_at[node]
    let at = cols[k].ys.add(low_bits(s.y))
    cols[k].colour.resize(cols[k].ys.len, 0)
    cols[k].colour[at] += s.v

for qu in queries:
    var total = 0
    for o in walk(xs, qu.r):
        if let k = column_at.get(o):
            total += sum(cols[k].colour[p] for p in walk(cols[k].ys, qu.c))
    print(total)
