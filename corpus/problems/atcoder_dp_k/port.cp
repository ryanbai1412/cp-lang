// Source: AtCoder Educational DP K (Stones), https://atcoder.jp/contests/dp/tasks/dp_k
// limits: 2000 ms, 1024 MB
// Problem: a pile of k <= 1e5 stones and a set of n <= 100 sizes 1 <= a_1 < ... < a_n <= k.
//   Players alternately remove exactly some a_i stones; whoever cannot move loses. Print First
//   if the first player wins, else Second.
// Solution: wins[s] (player to move with s stones wins) holds iff some a_i <= s leads to a
//   losing wins[s - a_i] = false; fill s = 0..k upward. O(nk).
// original: corpus/submissions/4131775.cpp
// Bounds: n: 1..=100; k: 1..=1e5; a: 1..=k
let n: 1..=100 = read()
let k: 1..=1e5 = read()
let a: [n]1..=k = read()

domain Stones = 0..=k
// wins[s]: the player to move with s stones left can force a win.
var wins: [Stones]Bool = false
for s in Stones:
    wins[s] = any(Stones.at(s - x).is_some_and(rest => not wins[rest]) for x in a)
print(if wins[Stones.last!]: "First" else: "Second")
