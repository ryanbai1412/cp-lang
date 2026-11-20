// Codeforces 2255 B: the number of maximal blocks of each digit is fixed, so
// for each digit distribute its surplus characters over its blocks (stars
// and bars) and multiply the two counts.
const MOD = 998244353
type Z = Mod<MOD>
let comb = Comb<MOD>()
@!multitest
let n: 1..=2e5 = read()
let s: Str = read()
var total = [0, 0]
var blocks = [0, 0]
for (c, len) in runs(s):
    total[c - '0'] += len
    blocks[c - '0'] += 1
var ans: Z = 1
for z in 0..2 if blocks[z] > 0: ans *= comb.choose(total[z] - 1, blocks[z] - 1)
print(ans)
