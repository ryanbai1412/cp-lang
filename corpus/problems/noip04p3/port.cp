// Source: NOIP 2004 P3 (Chorus Formation), https://dmoj.ca/problem/noip04p3
// Problem: N <= 100 heights in 130..=230 in a row. Remove the fewest students so the rest strictly
//   rise to one student and strictly fall after; print that count.
// Solution: up[i] is the longest strictly rising subsequence ending at i and down[i] the longest
//   strictly falling one starting there, each an O(N^2) LIS DP. A formation peaking at i keeps
//   up[i] + down[i] - 1; print N minus the best.
// original: corpus/submissions/5117143.cpp
let n: 2..=100 = read()
domain I = 1..=n
let h: [I]130..=230 = read()

var up: [I]Int = 1
var down: [I]Int = 1
for i in I:
    up[i] = 1 + ([up[j] for j in I if j < i and h[j] < h[i]].max() ?? 0)
for i in I.rev:
    down[i] = 1 + ([down[j] for j in I if j > i and h[j] < h[i]].max() ?? 0)
print(n - [up[i] + down[i] - 1 for i in I].max()!)
