// Source: COCI '21 Contest 1 #1 (Ljeto), https://dmoj.ca/problem/coci21c1p1
// Problem: n <= 100 sprays (t, a, b) in increasing time t; players 1-4 are the pineapples,
//   5-8 the blueberries, and a always sprays an opponent b. A spray scores 100 for a's team,
//   plus 50 if a also sprayed within the previous 10 seconds. Print both teams' totals.
// Solution: keep each player's last spray time (None before the first); a spray scores
//   150 if t - last <= 10, else 100, and goes to the team of the player number. O(n).
// original: corpus/submissions/4080824.cpp
let n: Int = read()
domain Player = 1..=8
var last: [Player]?Int = None
var first = 0
var second = 0
for _ in 0..n:
    let t: Int = read()
    let who: Player = read()
    let hit: Player = read()
    var points = 100
    if let before = last[who]:
        if t - before <= 10: points = 150
    if who.ord < 4: first += points
    else: second += points
    last[who] = t
print(first, second)
