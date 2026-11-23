// Codeforces 2255 B: A Ribbon for Tomorrow, tuned.
// port.cp's count, in one pass over the characters: a digit opens a block
// where it differs from the one before, instead of `s.runs()`, which builds
// a list of every run (it calls a fn value, so it can't feed the loop).

type Z = Mod<998_244_353>
let comb = Comb<Z>()
domain Bit = 0..2

@!multitest
let n: 1..=200_000 = read()
let s: Str = read()

var total: [Bit]Int = 0
var blocks: [Bit]Int = 0
for i in 0..s.len:
    let d = Bit(s[i] - '0')
    total[d] += 1
    if i == 0 or s[i] != s[i - 1]: blocks[d] += 1
print(fold(*, comb.choose(total[d] - 1, blocks[d] - 1) for d in Bit if blocks[d] > 0))
