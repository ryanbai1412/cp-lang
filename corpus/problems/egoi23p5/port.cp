// Source: EGOI 2023 Day 2 (Carnival General), https://dmoj.ca/problem/egoi23p5
// limits: 1000 ms, 1024 MB
// Problem: generals 0..n-1 (n <= 1000); general i ranks 0..i-1 from best to worst. Line them
//   all up so that for neighbours i < j, i is not strictly in the worse half of j's ranking.
//   Print any such order.
// Solution: insert generals in order 1..n-1, each liking the first ceil(i/2) of its ranking:
//   in front if it likes the first of the line, at the end if it likes the last, else
//   between two adjacent liked ones, which exist since ceil(i/2) liked sit among the i - 2
//   inner places. O(n^2).
// Bounds: n: 1..=1000; ranking: 0..n
// original: corpus/submissions/5678053.cpp

let n: 1..=1000 = read()
domain G = 0..n
var line: List<G> = [G.first!]
for me in G if me != G.first!:
    let ranking: [me.ord]G = read()
    var liked: [G]Bool = false
    for g in ranking[0..(me.ord + 1) / 2]: liked[g] = true
    let at =
        if liked[line.first!]: 0
        elif liked[line.last!]: line.len
        else: (0..line.len - 1).find(j => liked[line[j]] and liked[line[j + 1]])! + 1
    line.insert(at, me)
print(line)
