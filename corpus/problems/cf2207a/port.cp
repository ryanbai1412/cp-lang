// Source: Codeforces 2207A (1-1), https://codeforces.com/contest/2207/problem/A
// limits: 1000 ms, 256 MB
// Problem: a binary string s, 3 <= n <= 100. A move sets any s_i with s_{i-1} = s_{i+1} = 1 to 0
//   or 1. Print the least and the most 1s reachable. Multitest, t <= 500.
// Solution: the end 1s of each maximal run of 1s with single-0 gaps never change, so each such
//   block is independent. Filling every 0 between two 1s left to right gives the most; then
//   clearing every 1 between two 1s left to right leaves 1010...1, floor(L / 2) + 1 ones per
//   block of length L, the least since no move makes two adjacent 0s inside a block. O(n).
// Bounds: n: 3..=100

@!multitest
let n: 3..=100 = read()
var s: [0..n]Char = read()
fn sandwiched(i: Int) -> Bool: s[i] == '1' and s[i + 2] == '1'

for i in 0..n - 2 if sandwiched(i): s[i + 1] = '1'
let max_cnt = s.count(c => c == '1')
for i in 0..n - 2 if sandwiched(i): s[i + 1] = '0'
print(s.count(c => c == '1'), max_cnt)
