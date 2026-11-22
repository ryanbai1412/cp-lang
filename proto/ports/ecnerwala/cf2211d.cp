// Codeforces 2211 D — Binomial Bit-Count Recovery
//
// Each `b[i]` equals `sum_{bit=0..29} 2^bit * choose(cnts[bit], i) (mod 10^9 + 7)`,
// where `cnts[bit]` is the number of elements in the reconstructed array having
// `bit` set. Scanning `i` from `n` down to `1`:
// - Subtract `2^bit * choose(cnts[bit], i)` for every bit already known to have
//   `cnts[bit] > i`.
// - Any remaining bit set in `val.v` must have `cnts[bit] == i` (since
//   `choose(i, i) == 1` and all bits `< 29` sum to `< 2^29 < MOD`), so record
//   `cnts[bit] = i` and include `2^bit` in the current element `a`.

const MOD = 1_000_000_007
const L = 29
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let b: [1..=n]Z = read()

var cnts: [0..L]Int = 0
var out: List<Int> = []
for i in (1..=n).rev:
    var val = b[i]
    var a = 0
    for bit in 0..L if cnts[bit] > 0:
        val -= (1 << bit) * comb.choose(cnts[bit], i)
        a += 1 << bit
    if val != 0:
        for bit in 0..L if val.v.bit(bit):
            cnts[bit] = i
            a += 1 << bit
    out.push(a)
print(out)
