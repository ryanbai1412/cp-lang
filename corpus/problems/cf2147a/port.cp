// Source: Codeforces 2147A (Shortest Increasing Path),
//   https://codeforces.com/contest/2147/problem/A
// limits: 1000 ms, 256 MB
// Problem: walk from (0, 0) to (x, y) (1 <= x, y <= 1e9) with steps along +x, +y, +x, ... in turn,
//   starting with x, each of positive length strictly longer than the last. Print the fewest
//   steps, or -1. Multitest.
// Solution: if x < y, two steps (x, then y). Otherwise two steps fail and three need
//   x1 < y < x - x1 with x1 >= 1, possible iff y >= 2 and x - y >= 2 (take x1 = 1); longer walks
//   never help, so else -1. O(1).
// Bounds: x, y: 1..=10 ** 9

@!multitest
let (x, y): (1..=10 ** 9, 1..=10 ** 9) = read()
print(if x < y: 2 elif x - y <= 1 or y == 1: -1 else: 3)
