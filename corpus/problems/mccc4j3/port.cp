// Source: Mock CCC '22 Contest 2 J3 (Figure Skating Fun), https://dmoj.ca/problem/mccc4j3
// Problem: n <= 1e6 scores 1 <= s_i <= 100. Print the spot M (between competitors M and M + 1)
//   where the sum on the left equals the sum on the right, or "Andrew is sad." if none.
// Solution: scores are positive, so prefix sums increase and at most one is half the total:
//   scan for the first prefix with 2 * prefix == total. O(n).
// original: corpus/submissions/4264907.cpp

let n: Int = read()
domain V = 1..=n
let a: [V]Int = read()

var total = 0
for i in V: total += a[i]

var cur = 0
var ans: ?V = None
for i in V:
    cur += a[i]
    if 2 * cur == total:
        ans = i
        break

if let idx = ans: print(idx)
else: print("Andrew is sad.")
