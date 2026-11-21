// Codeforces 2201 E: ABBA Counting.
use lib.math.poly.{Z, convolve}
use std.math.{divisors, factor}

fn cyclic_convolve(a: List<Z>, b: List<Z>, len: Int) -> List<Z>:
    let prod = convolve(a, b)
    var res: [0..len]Z = 0
    for i in 0..prod.len: res[i % len] += prod[i]
    res

@!multitest
let n: Int = read()
let t: Str = read()
let m = n / 2

var pow2: [0..=m]Z = 1
for i in 1..=m: pow2[i] = pow2[i - 1] * 2

var ans: Z = 0
'divisor: for f in divisors(m):
    let len = m / f
    var s: [0..2][0..len]?Char = none
    for z in 0..2:
        for k in 0..f:
            let off = z * m + k * len
            for i in 0..len:
                let c = t[off + i]
                if c == '?': continue
                if let prev = s[z][i], prev != c: continue 'divisor
                s[z][i] = c

    var sign0: [0..len]Z = 0
    var sign1: [0..len]Z = 0
    var fix0: [0..len]Z = 0
    var fix1: [0..len]Z = 0
    var tot_fix = 0
    for i in 0..len:
        if let c = s[0][i]:
            sign0[i] = if c == 'a': 1 else: -1
            fix0[i] = 1
            tot_fix += 1
        if let c = s[1][len - 1 - i]:
            sign1[i] = if c == 'a': 1 else: -1
            fix1[i] = 1
            tot_fix += 1

    let diff_conv = cyclic_convolve(sign0, sign1, len)
    let both_conv = cyclic_convolve(fix0, fix1, len)
    var ways: Z = 0
    for i in 0..len:
        if diff_conv[i] == both_conv[i]:
            ways += pow2[len - tot_fix + both_conv[i].v]

    var w: Z = 1
    for (p, _) in factor(f): w *= 1 - p
    ans += ways * w

print(ans)
