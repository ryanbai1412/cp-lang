// Source: DMOPC 2020 Contest 4 P4 (Javelin Throwing), https://dmoj.ca/problem/dmopc20c4p4
// Problem: n <= 2e6 throws each make a hole on a line unless landing in an existing one. After
//   throw i you learn 0 <= a_i < i, the holes strictly behind it; b_i counts those strictly in
//   front. Print, space-separated, the least possible sum of b_1..b_i given a_1..a_i, for every i.
// Solution: only the hole count h before a throw matters: a new hole costs b = h - a, landing in
//   an existing one b = h - 1 - a. Greedily land whenever a < h (cost h - a - 1) and keep the
//   throw as spare. When a exceeds h, some spare j must have been a new hole: that adds 1 to b_j
//   and to every b between, i - j in all, so reopen the latest spare. Amortised O(n).
// original: corpus/submissions/3485716.cpp
let n: 1..=2e6 = read()
var have = 0
var cost = 0
var spare: List<Int> = []
var out: List<Int> = []
for i in 0..n:
    let a: 0..=i = read()
    while have < a:
        cost += i - spare.pop()!
        have += 1
    if a == have:
        have += 1
    else:
        cost += have - a - 1
        spare.push(i)
    out.push(cost)
print(out)
