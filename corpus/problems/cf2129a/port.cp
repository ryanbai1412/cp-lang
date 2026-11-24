// Source: Codeforces 2129A (Double Perspective), https://codeforces.com/contest/2129/problem/A
// limits: 2000 ms, 256 MB
// Problem: n <= 3000 distinct pairs a < b <= 2n. For a subset, f is the length of the union of the
//   segments [a, b], g the number of nodes on a simple cycle (>= 3 edges) of the graph with the
//   pairs as edges. Print a subset maximising f - g: its size and its indices. Multitest.
// Solution: for each left endpoint keep only the pair with the largest right end. The union is
//   unchanged (each pair lies inside the kept one with its left end), and distinct left ends make
//   the graph a forest, so g = 0. One `max=` per pair into best_end over positions: O(n).

@!multitest
let n: 1..=3_000 = read()
domain Pos = 1..=2 * n
domain Id = 1..=n
var best_end: [Pos]?(end: Pos, id: Id) = None
for id in Id:
    let (a, b): (Pos, Pos) = read()
    best_end[a] max= (end: b, id:)

let ans = [p.id for x in best_end if let p = x]
print(ans.len)
print(ans)
