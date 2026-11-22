// AtCoder AGC 079 A: value i must come after everything in p[i]..i, which
// makes a forest; the count of valid orders is n! over the product of subtree
// sizes. A stack of (rightmost index, size) merges each subtree as it closes.
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
