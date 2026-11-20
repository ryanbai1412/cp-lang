// COCI 2021 #6 P5 Zagrade-style pairing: a word of 2n brackets is split into
// n pairs of positions (x < y). Keep exactly one bracket of every pair so
// that the kept ones, in order, are balanced. Print for each pair whether
// its second bracket is the one kept, or -1.
//
// Greedy: a pair of equal brackets has one best choice (the earlier '(' or
// the later ')'). A mixed pair starts on its ')' and can be flipped to its
// '(' later; when the running depth goes negative, flip the open mixed pair
// that closes earliest.
// original: corpus/submissions/4531869.cpp
use std.heap.{MinHeap}

fn solve():
    let n: Int = read()
    let s: Str = read()
    var pairs: List<(Int, Int)> = []
    for _ in 0..n:
        let (x, y): (Int, Int) = read()
        pairs.push((x - 1, y - 1))

    var kept = [false for _ in 0..2 * n]
    var opens_mixed: List<?Int> = [none for _ in 0..2 * n]      // at x: the mixed pair starting there
    for i in 0..n:
        let (x, y) = pairs[i]
        if s[x] == s[y]:
            kept[if s[x] == '(': x else: y] = true
        else:
            opens_mixed[x] = i
            kept[if s[x] == ')': x else: y] = true

    var depth = 0
    var flippable = MinHeap<(Int, Int)>()                       // (y, pair)
    for i in 0..2 * n:
        if let p = opens_mixed[i]: flippable.push((pairs[p].1, p))
        if not kept[i]: continue
        depth += if s[i] == '(': 1 else: -1
        if depth >= 0: continue
        let (_, p) = flippable.pop() else:
            print(-1)
            return
        let (x, y) = pairs[p]
        kept.swap(x, y)
        depth += if y <= i: 2 else: 1
    if depth != 0:
        print(-1)
        return
    var line = Str()                                            // one "0 1 1 0" row
    for (_, y) in pairs:
        if line.len > 0: line.push(' ')
        line.push(if kept[y]: '1' else: '0')
    print(line)

let t: Int = read()
for _ in 0..t: solve()
