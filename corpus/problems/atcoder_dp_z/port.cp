// Source: AtCoder Educational DP Contest Z (Frog 3), https://atcoder.jp/contests/dp/tasks/dp_z
// limits: 2000 ms, 1024 MB
// Problem: n <= 2e5 stones with heights 1 <= h_1 < ... < h_n <= 1e6. A jump from stone i to
//   any later stone j costs (h_j - h_i)^2 + C, C <= 1e12. Print the cheapest way from stone
//   1 to stone n.
// Solution: cost[i] = C + h[i]^2 + min over j < i of (cost[j] + h[j]^2) - 2 h[j] h[i], the
//   lowest of a set of lines at x = h[i]. A `LiChao` over the heights 1..=1e6 keeps that lower
//   envelope: O(n log 1e6).
// original: corpus/submissions/4133674.cpp
// Bounds: n: 2..=2e5; c: 1..=1e12; h: 1..=1e6
let n: 2..=2e5 = read()
let c: 1..=1e12 = read()
domain Stone = 1..=n
let h: [Stone]1..=1e6 = read()

var lines = LiChao(min, over: 1..=1e6)
var cost: [Stone]Int = 0
for s in Stone:
    if s != Stone.first!: cost[s] = lines.at(h[s])! + h[s] * h[s] + c
    lines.add(Line(-2 * h[s], cost[s] + h[s] * h[s]))
print(cost[Stone.last!])
