// Source: CCO 2017 Day 1 P3 (Vera and Modern Art), https://dmoj.ca/problem/cco17p3
// limits: 4000 ms, 1024 MB
// Problem: N <= 2e5 strokes x y v (x, y <= 1e18, v <= 1e4): with a, b the largest powers of
//   two not above x and y, add v to every point (x + a p, y + b q), p, q >= 0. Then answer
//   Q <= 2e5 queries: the total colour at point (r, c), r, c <= 1e18.
// Solution: write x as a leading 1 and k low bits; a stroke reaches row r exactly when r
//   has the same k low bits and more than k bits. So insert each x's low bits, lowest
//   first, into a binary trie, each x node owning a binary trie over y holding the colour
//   sums. A query walks r's bits down the x trie and, at every node on that path, walks c's
//   bits down its y trie adding the totals: O(60) per stroke, O(60^2) per query.
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
