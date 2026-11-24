// Source: ETSR 2023 Round 1 P2, https://dmoj.ca/problem/etsr23r1p2
// Problem: a set of ranges on 1..=n (n <= 1e9) starts empty. Of q <= 2e5 orders, kind 2 adds
//   l..=r after throwing out every range in the set that overlaps it, and prints how many it
//   threw out; kind 1 prints how many ranges the set holds.
// Solution: the set's ranges never overlap, so ordered by start they are ordered by end too. A
//   SortedBag of (start, end) holds them: one cursor search finds the last range starting before
//   l, which overlaps if it reaches l, and stepping on from it walks every range starting in l..=r.
//   Each range is thrown out at most once, so O(q log q).
// original: corpus/submissions/5494248.cpp
// Bounds: q: 1..=2e5; n: 1..=1e9; kind: 1..=2; l: 1..=n; r: l..=n
let q: 1..=2e5 = read()
let n: 1..=1e9 = read()
var held = SortedBag<(Int, Int)>()
for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        print(held.len)
        continue
    let l: 1..=n = read()
    let r: l..=n = read()
    let before = held.cursor_lt((l, 0))
    var gone: List<(Int, Int)> = []
    if let b = before and held.key_of(b).1 >= l: gone.push(held.key_of(b))
    var c = if let b = before: held.next_cursor(b) else: held.cursor_first()
    while let x = c and held.key_of(x).0 <= r:
        gone.push(held.key_of(x))
        c = held.next_cursor(x)
    for k in gone: held.remove(k)
    held.add((l, r))
    print(gone.len)
