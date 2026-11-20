// Codeforces 2211 D: recover bit counts from i = N down to 1 by subtracting
// binomial contributions of already-active bits modulo 10^9 + 7.
const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<MOD>()

@!multitest
let n: Int = read()
let b: [1..=n]Z = read()

const L = 29
var cnts: [0..L]Int = 0
var out: List<Int> = []
for i in (1..=n).rev:
    var val = b[i]
    var a = 0
    for bit in 0..L:
        if cnts[bit] > 0:
            val -= Z(1 << bit) * comb.choose(cnts[bit], i)
            a += 1 << bit
    if val != 0:
        let v = val.v
        for bit in 0..L:
            if v.bit(bit):
                cnts[bit] = i
                a += 1 << bit
    out.push(a)
print(out)
