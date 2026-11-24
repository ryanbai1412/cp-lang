// Source: EGOI 2023 Day 2 Problem 2 (Candy), https://dmoj.ca/problem/egoi23p6
// limits: 3000 ms, 1024 MB
// Problem: n <= 100 boxes with a[i] <= 1e9 candy, f <= n and 0 <= t <= 1e11. Print the fewest swaps
//   of adjacent boxes after which the first f boxes hold at least t candy, or NO.
// Solution: moving chosen boxes j_0 < ... < j_(f-1) to the front costs sum (j_i - i) swaps, at
//   most f (n - f) for the last f boxes. best[i, j, s]: most candy from boxes j.. put into slots
//   i..f-1 with at most s swaps; skip box j, or put it in slot i for j - i swaps. The answer is
//   the least s with best[0, 0, s] >= t: O(f n * f (n - f)) <= 2.5e7.
// original: corpus/submissions/5678056.cpp
// Bounds: n: 1..=100; f: 1..=n; t: 0..=1e11; candy: 0..=1e9

let n: 1..=100 = read()
let f: 1..=n = read()
let t: 0..=1e11 = read()
let candy: [n]0..=1e9 = read()

let cap = f * (n - f)
var best: [0..=f, 0..=n, 0..=cap]Int = 0
for i in (0..f).rev:
    for j in (i..n).rev:
        for s in (j - i)..=cap:
            best[i, j, s] = max(best[i, j + 1, s], candy[j] + best[i + 1, j + 1, s - (j - i)])

if let s = (0..=cap).find(s => best[0, 0, s] >= t): print(s)
else: print("NO")
