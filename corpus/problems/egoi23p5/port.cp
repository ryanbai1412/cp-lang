// Source: EGOI 2023 Day 2 (Carnival General), https://dmoj.ca/problem/egoi23p5
// Problem: generals 0..n-1 (n <= 1000); general i ranks 0..i-1 from best to worst. Line them
//   all up so that for neighbours i < j, i is not strictly in the worse half of j's ranking.
//   Print any such order.
// Solution: insert generals in order 1..n-1, each liking the first ceil(i/2) of its ranking:
//   in front if it likes the first of the line, at the end if it likes the last, else
//   between two adjacent liked ones, which exist since ceil(i/2) liked sit among the i - 2
//   inner places. O(n^2).
// Note: runnable on proto/cpc today.
// original: corpus/submissions/5678053.cpp
// Bounds: n: 1..=1000
let n: 1..=1000 = read()
domain G = 0..n
var path: List<G> = [G(0)]
for i in 1..n:
    var liked: [G]Bool = false
    for j in 0..i:
        let g: G = read()
        if j < (i + 1) / 2: liked[g] = true
    let me = G(i)
    var at = path.len
    if liked[path[0]]: at = 0
    elif not liked[path.last!]:
        for j in (0..path.len - 1).rev:
            if liked[path[j]] and liked[path[j + 1]]: at = j + 1
    var next: List<G> = []
    for j in 0..path.len:
        if j == at: next.push(me)
        next.push(path[j])
    if at == path.len: next.push(me)
    path = next
print(path)
