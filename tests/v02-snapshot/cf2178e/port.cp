// Source: Codeforces 2178E (Flatten or Concatenate), https://codeforces.com/contest/2178/problem/E
// Problem: interactive. A hidden array of n <= 1e5 elements (1..=2^30) was built from [2^k] by
//   flattening a maximal x into x/2, x/2 and by concatenations a := a + b. With at most 300
//   range-sum queries, find its maximum. Multitest.
// Solution: any block of length >= 2 is the concatenation of two halves of equal sum, and the
//   shorter half holds a maximum. Binary search for the first prefix with sum >= half the
//   total to find the split, then move into the shorter half, which at least halves the length:
//   O(log^2 n) queries; the last block's sum is the answer.

@!interactive

@!multitest
let n: 1..=100_000 = read()
domain Pos = 1..=n

fn ask(lo: Pos, hi: Pos) -> Int:
    print("?", lo, hi)
    read()

var l = Pos(1)
var r = Pos(n)
var tot = ask(l, r)
while l < r:
    let mid = (l..=r.prev!).first_true(md => 2 * ask(l, md) >= tot) ?? r.prev!
    tot /= 2
    if r.ord - mid.ord <= mid.ord - l.ord + 1:
        l = mid.next!
    else:
        r = mid
print("!", tot)
