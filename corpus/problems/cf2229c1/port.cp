// Source: Codeforces 2229C1 (We Be Flipping, Easy Version),
//   https://codeforces.com/contest/2229/problem/C1
// limits: 2000 ms, 256 MB
// Problem: n <= 2e5 non-zero integers, |a_i| <= 1e9. An operation picks i with a_i > 0 and negates
//   a_1..a_i. Print at most n operations (count, then indices) leaving the least possible sum.
//   Multitest, sum of n <= 2e5.
// Solution: every element can end negative. Sweep i from n down with `flip`, the parity of the
//   operations so far (all of which cover i): if a_i is positive now, operate at i, which makes it
//   negative; later operations stop below i. At most n operations, O(n).
// Bounds: n: 1..=200_000

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
let a: [Pos]-10 ** 9..=10 ** 9 = read()

var ops: List<Pos> = []
var flip = false
for i in Pos.rev:
    if (a[i] > 0) != flip:
        ops.push(i)
        flip = not flip
print(ops.len)
print(ops)
