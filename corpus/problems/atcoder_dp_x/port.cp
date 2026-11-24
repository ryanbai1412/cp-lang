// Source: AtCoder Educational DP Contest X (Tower), https://atcoder.jp/contests/dp/tasks/dp_x
// limits: 2000 ms, 1024 MB
// Problem: N <= 1000 blocks, block i with weight w_i <= 1e4, solidness s_i <= 1e4 and value
//   v_i <= 1e9. Stack some of them so every block carries a total weight above it of at most
//   its s; print the largest total value of the tower.
// Solution: exchange argument: some optimal tower has s + w non-decreasing from the top, so
//   take the blocks in that order and build top down. best[x] is the most value of a tower
//   weighing x; block b goes under any tower with x <= s_b, a 0/1 knapsack scanned downward
//   over x. A tower never weighs more than max(s + w) <= 2e4. O(N * max s).
// original: corpus/submissions/4133420.cpp
let n: 1..=1000 = read()
let blocks: [n](w: 1..=1e4, s: 1..=1e4, v: 1..=1e9) = read()

domain Weight = 0..=2e4
var best: [Weight]?Int = None
best[Weight.first!] = 0
for b in blocks.sorted(by: b => b.s + b.w):
    for x in (..=Weight(b.s)).rev:
        if let v = best[x]: best[Weight(x.label + b.w)] max= v + b.v
print(best.map(v => v ?? 0).max()!)
