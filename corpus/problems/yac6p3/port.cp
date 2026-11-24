// Source: Yet Another Contest 6 P3 (No More Cell Phone Messaging),
//   https://dmoj.ca/problem/yac6p3
// Problem: interactive, non-adaptive: two distinct hidden cells (A, B), (C, D) of an N x M grid
//   (N, M <= 1e9). Query "? X Y" returns min(|X - A| + |Y - B|, |X - C| + |Y - D|); after at
//   most 5 queries print "! A B C D" (either order).
// Solution: the four corners give min(x + y), min(y - x), min(x - y) and max(x + y) over the
//   two cells. Either the min-sum cell also has the min y - x (the other then has the max sum
//   and the min x - y) or the pairing is crossed; a fifth query at the first pairing's
//   candidate cell tells which, and each cell follows from its sum and difference. O(1).
// As it builds today: the four candidate cells are plain numbers checked by hand (K16).
// original: corpus/submissions/5149597.cpp; judge: yac6p3_judge.py
// Bounds: n, m: 1..=1e9
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
domain X = 1..=n
domain Y = 1..=m

fn near(x: X, y: Y) -> Int:
    let d: 0..=2e9 = ask("?", x, y)
    d

let a = near(X.first!, Y.first!) + 2
let b = near(X.last!, Y.first!) - n + 1
let c = near(X.first!, Y.last!) - m + 1
let d = n + m - near(X.last!, Y.last!)

var lower_pair = false
let y = (a + b) / 2
let x = a - y
if (a + b) % 2 == 0 and 1 <= x <= n and 1 <= y <= m:
    if near(X(x), Y(y)) == 0: lower_pair = true
if lower_pair: print("!", x, y, d - (d - c) / 2, (d - c) / 2)
else: print("!", (a + c) / 2, a - (a + c) / 2, (d - b) / 2, d - (d - b) / 2)
