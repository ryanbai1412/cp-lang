// CCO 2017 #3 Vera and Modern Art: an endless grid of zeros. A stroke
// x y v adds v to the cells (x + i a, y + j b) for all i, j >= 0, where a
// is the largest power of two not above x and b the same for y. After n
// strokes, print the value of each of q cells.
//
// Write x as a leading 1 followed by k low bits. The stroke reaches row r
// exactly when r has the same k low bits and at least k + 1 bits. Reading
// each coordinate from its lowest bit upwards gives a binary trie over x
// where each node optionally owns a binary trie over y.
use std.strings.{Trie}

domain Bit = 0..2

fn bits_of(x: Int) -> List<Bit>:
    var out: List<Bit> = []
    var rest = x
    while rest > 1:
        out.push(Bit(rest % 2))
        rest /= 2
    out

fn path_sum(t: Trie<Bit>, total: List<Int>, y: Int) -> Int:
    var o = t.root
    var ans = total[o]
    var rest = y
    while rest > 1:
        if let next = t.step(o, Bit(rest % 2)):
            o = next
            ans += total[o]
            rest /= 2
        else:
            break
    ans

let (n, q): (Int, Int) = read()
var xs = Trie<Bit>.new()
var y_id: List<?Int> = [None]
var y_tries: List<Trie<Bit>> = []
var y_totals: List<List<Int>> = []

for _ in 0..n:
    let (x, y, v): (Int, Int, Int) = read()
    let col = xs.add(bits_of(x))
    while y_id.len < xs.len: y_id.push(None)
    if y_id[col] == None:
        y_id[col] = y_tries.len
        y_tries.push(Trie<Bit>.new())
        y_totals.push([0])
    let tid = y_id[col]!
    let yn = y_tries[tid].add(bits_of(y))
    while y_totals[tid].len < y_tries[tid].len: y_totals[tid].push(0)
    y_totals[tid][yn] += v

for _ in 0..q:
    let (r, c): (Int, Int) = read()
    var o = xs.root
    var ans = if let tid = y_id[o]: path_sum(y_tries[tid], y_totals[tid], c) else: 0
    var rest = r
    while rest > 1:
        if let next = xs.step(o, Bit(rest % 2)):
            o = next
            if let tid = y_id[o]: ans += path_sum(y_tries[tid], y_totals[tid], c)
            rest /= 2
        else:
            break
    print(ans)
