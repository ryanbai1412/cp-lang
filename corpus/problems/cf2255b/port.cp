// Source: Codeforces 2255B (A Ribbon for Tomorrow), https://codeforces.com/contest/2255/problem/B
// Problem: multitest (t <= 1e4, sum n <= 1e6): a binary string s of length n <= 1e6. An
//   operation picks l <= r with s[l] = s[r] and reverses s[l..r]. Print how many distinct
//   strings can be reached, mod 998244353.
// Solution: an operation keeps, per digit, its total count and its number of maximal runs,
//   and every way to share a digit's total among that many non-empty runs is reachable.
//   By stars and bars that is C(total - 1, blocks - 1) per digit; the answer is the product
//   over the digits that occur. One pass over `s.runs()`, O(n).

type Z = Mod<998_244_353>
let comb = Comb<Z>()
domain Bit = 0..2

@!multitest
let n: 1..=1e6 = read()
let s: Str of "01" = read()
assert s.len == n

var total: [Bit]Int = 0
var blocks: [Bit]Int = 0
for run in s.runs():
    let d = Bit(run.first! - '0')
    total[d] += run.len
    blocks[d] += 1
print(fold(*, comb.choose(total[d] - 1, blocks[d] - 1) for d in Bit if blocks[d] > 0))
