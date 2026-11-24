// Source: EGOI 2023 Day 2 Problem 2 (Candy), https://dmoj.ca/problem/egoi23p6
// Problem: n <= 100 boxes with a[i] <= 1e9 candy, f <= n and 0 <= t <= 1e11. Print the fewest swaps
//   of adjacent boxes after which the first f boxes hold at least t candy, or NO.
// Solution: moving chosen boxes j_0 < ... < j_(f-1) to the front costs sum (j_i - i) swaps.
//   best[i, j, s]: most candy from boxes j.. put into slots i..f-1 with at most s swaps; skip
//   box j, or put it in slot i for j - i swaps. Swaps never need to exceed n^2 / 2, so the
//   answer is the least s with best[0, 0, s] >= t: O(f n * n^2/2) = 5e7.
// original: corpus/submissions/5678056.cpp
// Runnable on proto/cpc today.
let n: 1..=100 = read()
let m: 1..=n = read()
let k: 0..=1e11 = read()
domain Box = 0..n
let a: [Box]0..=1e9 = read()

let cap = n * n / 2 + 10
domain Take = 0..=m
domain Pos = 0..=n
domain Swaps = 0..cap
// best[i, j, s]: most candy from boxes j.. placed into slots i..m-1 with s swaps
var best: [Take, Pos, Swaps]Int = 0
for i in (0..m).rev:
    for j in (i..n).rev:
        for s in (j - i)..cap:
            best[Take(i), Pos(j), Swaps(s)] = max(
                best[Take(i), Pos(j + 1), Swaps(s)],
                a[Box(j)] + best[Take(i + 1), Pos(j + 1), Swaps(s - (j - i))],
            )

var ans: ?Int = None
for s in 0..cap:
    if ans == None and best[Take(0), Pos(0), Swaps(s)] >= k: ans = s
if let s = ans: print(s)
else: print("NO")
