// Codeforces 2180 H2 — Bug Is Feature (Hard Version)
//
// Same interval-XOR Sprague-Grundy game as H1 (`cf2180h1`): each tuple
// (a, b, c, l, r) with common difference d = c - b = b - a contributes the
// XOR of Grundy values over x in [l, r], evaluated as prefix_xor(r - c + 1, d)
// ^ prefix_xor(l - c, d).
//
// In H2 the closed form splits on the parity of d:
// - Odd d: block reduction collapses the prefix to g_hi = len / d + ((len % d) & 1) + 1.
//   The alternating bit-weight `popcount(g_hi) - 2 * popcount(g_hi & EVEN_BITS_FROM_2)`
//   tracks the net odd-minus-even 1-bits (above bit 0), indexing the 4-cycle
//   ODD_D_GRUNDY = [0, 2, 3, 1].
// - Even d: even prefix lengths cancel to 0; odd lengths collapse to
//   g = len / d + 2, giving 0 when ctz(g) is odd, else 2 (odd popcount) or 1.

const EVEN_BITS_FROM_2 = 0x5555555555555554
const ODD_D_GRUNDY = [0, 2, 3, 1]

fn prefix_xor(len: Int, d: Int) -> Int:
    if d % 2 == 1:
        let g_hi = len / d + ((len % d) & 1) + 1
        let alt = popcount(g_hi) - 2 * popcount(g_hi & EVEN_BITS_FROM_2)
        ODD_D_GRUNDY[alt & 3]
    elif len % 2 == 0:
        0
    else:
        let g = len / d + 2
        if ctz(g) % 2 == 1: 0
        elif popcount(g) % 2 == 1: 2
        else: 1

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
