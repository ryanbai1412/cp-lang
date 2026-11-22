// Codeforces 2255 B — Binary Block Redistribution
//
// The number of maximal runs (`blocks[d]`) and total occurrences (`total[d]`)
// of each binary digit `d in Bit` are invariant. Distributing the `total[d]`
// identical copies across `blocks[d]` non-empty runs has `C(total[d] - 1, blocks[d] - 1)`
// ways by stars and bars, and the two digits are independent, so the answer
// is the product over digits with `blocks[d] > 0`.

type Z = Mod<998_244_353>
let comb = Comb<Z>()
domain Bit = 0..2

@!multitest
let n: 1..=200_000 = read()
let s: [0..n]Char = read()

var total: [Bit]Int = 0
var blocks: [Bit]Int = 0
for (c, len) in runs(s):
    let d = Bit(c - '0')
    total[d] += len
    blocks[d] += 1
print(fold(*, comb.choose(total[d] - 1, blocks[d] - 1) for d in Bit if blocks[d] > 0))
