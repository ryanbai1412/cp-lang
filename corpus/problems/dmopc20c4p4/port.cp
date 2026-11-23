// Source: DMOPC 2020 Contest 4 P4 (Javelin Throwing), https://dmoj.ca/problem/dmopc20c4p4
// Problem: n <= 2e6 throws each make a hole on a line unless landing in an existing one. After
//   throw i you learn a_i < i, the holes strictly behind it; b_i counts those strictly in front.
//   For every i print the least possible sum of b_1..b_i given a_1..a_i.
// Solution: greedy over the holes so far, `have`: a_i = have is a new frontmost hole (cost 0);
//   a_i < have lands in an existing hole, costing have - a_i - 1, and the throw is kept on a
//   stack as spare. When a_i exceeds have, the latest spare throw j must have made a new hole
//   instead: one more hole, charged i - j. Amortised O(n).
// original: corpus/submissions/3485716.cpp
let n: Int = read()
var have = 0
var ans = 0
var spare: List<Int> = []
var out: List<Int> = []
for i in 0..n:
    let a: Int = read()
    while have < a:
        let j = spare.last!
        spare.pop()
        ans += i - j
        have += 1
    if a == have:
        have += 1
    else:
        ans += have - a - 1
        spare.push(i)
    out.push(ans)
for x in out: print(x)
