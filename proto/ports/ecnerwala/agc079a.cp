// AtCoder AGC 079 A: value i must come after everything in p[i]..i, which
// makes a forest; the count of valid orders is n! over the product of subtree
// sizes. A stack of (rightmost index, size) merges each subtree as it closes.
const P = 998244353
type Z = Mod<P>
@!multitest
let n: 2..=3e5 = read()
let p: [0..n - 1]1..=n = read()
var blocks = List<(r: Int, size: Int)>()
var den = Z(1)
for i in 0..n - 1:
    var size = 1
    while blocks.len > 0 and blocks.last!.r >= p[i] - 1: size += blocks.pop()!.size
    den *= Z(size)
    blocks.push((r: i, size: size))
var num = Z(1)
for i in 1..n: num *= Z(i)
print(num / den)
