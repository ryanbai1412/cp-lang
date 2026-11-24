// Source: NOIP 2005 P2 (Crossing the River), https://dmoj.ca/problem/noip05p2
// Problem: a frog starts at 0 on a bridge of length L <= 1e9 and jumps s..=t (1 <= s <= t <= 10)
//   until it reaches or passes L. m <= 100 distinct points hold stones; print the fewest stones
//   it must land on.
// Solution: 2520 = lcm(1..=10), so a gap between stones longer than 2520 can shrink to
//   gap % 2520 + 2520 without changing what is reachable past it. That leaves under 5040 m
//   points; fewest[p], the stones landed on from p onwards, is stone[p] plus the min of
//   fewest[p + hop] over the hops, filled right to left. O(2520 m t).
// original: corpus/submissions/5156196.cpp
// Bounds: len: 1..=1_000_000_000; s: 1..=10; t: s..=10; m: 1..=100; stones: 1..=len - 1
const CYCLE = 2520
let len: 1..=1_000_000_000 = read()
let s: 1..=10 = read()
let t: s..=10 = read()
let m: 1..=100 = read()
let stones: [m]1..len = read()

var spot: List<Int> = []  // the stones, moved closer together
var (before, shrunk) = (0, 0)
for x in stones.sorted():
    let gap = x - before
    shrunk += if gap > CYCLE: gap % CYCLE + CYCLE else: gap
    spot.push(shrunk)
    before = x
let end = shrunk + 1  // past the last stone nothing matters

var stone: [0..end]Bool = false
for x in spot: stone[x] = true
// fewest[p]: stones landed on from p onwards, p included
var fewest: [0..end + t]Int = 0
for p in (0..end).rev:
    fewest[p] = min(fewest[p + hop] for hop in s..=t)! + Int(stone[p])
print(fewest[0])
