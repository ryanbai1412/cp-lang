// Codeforces 2180 H1 — Bug Is Feature (Easy Version)
//
// Each tuple (a, b, c, l, r) with b - a = c - b = d contributes one impartial
// game per pile size x in [l, r]; by Sprague-Grundy, "Bug" wins iff the XOR
// over all intervals is non-zero, computed as prefix_xor(r - c + 1, d) ^
// prefix_xor(l - c, d). (H2 `cf2180h2` is the same prefix-XOR game with a
// different move set and Grundy closed form.)
//
// Write d0 = 2^l * d with d odd (l = ctz(d0)). Partitioning offsets into
// blocks of size d collapses the prefix length to g_hi = n / d + ((n % d) & 1).
// The Grundy prefix XOR has period 2^(l+2): bit (l + 1) of g_hi toggles bit 0
// of the result, leaving the residue r = g_hi & ((2 << l) - 1) in 0..2^(l+1).
// For r == 0 the prefix XOR is just that bit; for r > 0 it flips bit 0 once,
// and when r is odd the half-period (r >= 2^l vs r < 2^l) and highest bit i
// of the distance to the nearest multiple of 2^(l+1) determine the remaining
// 2-bit correction via the parity of (l ^ i).

fn prefix_xor(len: Int, d0: Int) -> Int:
    let l = ctz(d0)
    let d = d0 >> l
    let half_period = 2 << l
    let period_mask = half_period - 1
    let g_hi = len / d + ((len % d) & 1)
    var res = Int(g_hi.bit(l + 1))
    let rem = g_hi & period_mask
    if rem == 0: return res
    res ^= 1
    if rem % 2 == 1:
        if rem.bit(l):
            let i = bit_width(half_period - rem) - 1
            res ^= if ((l ^ i) & 1) == 1: 2 else: 1
        else:
            let i = bit_width(rem) - 1
            res ^= if ((l ^ i) & 1) == 1: 0 else: 1
    res

@!multitest
let n: 1..=200_000 = read()
let qs: [0..n](
    a: 1..=10 ** 18,
    b: 1..=10 ** 18,
    c: 1..=10 ** 18,
    l: 1..=10 ** 18,
    r: 1..=10 ** 18,
) = read()
let ans = fold(^, prefix_xor(r - c + 1, c - b) ^ prefix_xor(l - c, c - b) for (_, b, c, l, r) in qs)
print(if ans != 0: "Bug" else: "Feature")
