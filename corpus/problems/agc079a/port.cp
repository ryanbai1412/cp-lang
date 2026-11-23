// Source: AtCoder AGC 079 A, https://atcoder.jp/contests/agc079/tasks/agc079_a
// Problem: a rooted tree on n <= 2.5e5 vertices, parent P_v < v; vertex v holds the sequence (v).
//   n - 1 times pick an unpicked v >= 2, append its sequence to its current parent's and
//   contract v. Count the pick orders leaving (1, 2, ..., n) at the root, mod 998244353.
//   Multitest, sum of n <= 2.5e5.
// Solution: v must be picked after every w with P_v < w < v; these constraints form a forest,
//   whose linear extensions number (n - 1)! / (product of subtree sizes). Scanning v upward, a
//   stack of (last vertex, size) blocks pops every block above P_v into v's subtree, whose size
//   joins the product. O(n).
const P = 998_244_353
type Z = Mod<P>
let comb = Comb<Z>()

@!multitest
let n: 2..=3e5 = read()
let p: [0..n - 1]1..=n = read()

var blocks: List<(r: Int, size: Int)> = []
var den: Z = 1
for (i, pi) in p.items:
    var size = 1
    while let b = blocks.last and b.r >= pi - 1:
        size += b.size
        blocks.pop()
    den *= size
    blocks.push((r: i, size:))
print(comb.fact(n - 1) / den)
