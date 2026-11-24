// Source: Mock CCC '22 Contest 2 J3 (Figure Skating Fun), https://dmoj.ca/problem/mccc4j3
// limits: 1000 ms, 256 MB
// Problem: n <= 1e6 scores 1 <= s_i <= 100. Print the spot M (between competitors M and M + 1)
//   where the sum on the left equals the sum on the right, or "Andrew is sad." if none.
// Solution: scores are positive, so prefix sums increase and at most one is half the total:
//   scan for the first prefix with 2 * prefix == total. O(n).
// Bounds: n: 1..=1e6; score: 1..=100
// original: corpus/submissions/4264907.cpp
let n: 1..=1_000_000 = read()
domain V = 1..=n
let score: [V]1..=100 = read()
let before = Prefix<V, Int>(score)
let total = score.sum()
if let spot = V.find(i => 2 * before.fold(V.first!..=i) == total): print(spot)
else: print("Andrew is sad.")
