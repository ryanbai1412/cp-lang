// Source: ETSR 2023 Round 1 P2, https://dmoj.ca/problem/etsr23r1p2
// Problem: a set of ranges on 1..=n (n <= 1e9) starts empty. Of q <= 2e5 orders, kind 2 adds
//   l..=r after throwing out every range in the set that overlaps it, and prints how many it
//   threw out; kind 1 prints how many ranges the set holds.
// Solution: the set's ranges never overlap, so ordered by start they are ordered by end too. A
//   SortedMap from start to end holds them: the ranges overlapping l..=r are the last one
//   starting before l, if it reaches l, and every one starting in l..=r. Each range is thrown
//   out at most once, so O(q log q).
// original: corpus/submissions/5494248.cpp
// Bounds: q: 1..=2e5; n: 1..=1e9; kind: 1..=2; l: 1..=n; r: l..=n
let q: 1..=2e5 = read()
let n: 1..=1e9 = read()
var held = SortedMap<Int, Int>()  // start -> end
for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        print(held.len)
        continue
    let l: 1..=n = read()
    let r: l..=n = read()
    var gone = held.drain(after: l - 1, while: (start, _) => start <= r).len
    if let (start, end) = held.last_lt(l):
        if end >= l:
            held.remove(start)
            gone += 1
    held.set(l, r)
    print(gone)
