// Source: AtCoder Educational DP K (Stones), https://atcoder.jp/contests/dp/tasks/dp_k
// Problem: a pile of k <= 1e5 stones and a set of n <= 100 sizes 1 <= a_1 < ... < a_n <= k.
//   Players alternately remove exactly some a_i stones; whoever cannot move loses. Print First
//   if the first player wins, else Second.
// Solution: wins[s] (player to move with s stones wins) holds iff some a_i <= s leads to a
//   losing wins[s - a_i] = false; fill s = 0..k upward. O(nk).
// original: corpus/submissions/4131775.cpp
let n: 1..=100 = read()
let k: 1..=1e5 = read()
let a: [n]1..=k = read()

// wins[s]: the player to move with s stones left can force a win.
var wins: [0..=k]Bool = false
for s in 0..=k:
    wins[s] = any(s >= x and not wins[s - x] for x in a)
print(if wins[k]: "First" else: "Second")
