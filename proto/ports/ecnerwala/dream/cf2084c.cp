// Codeforces 2084 C: simultaneously swap indices in two permutations a and b
// until a becomes the reverse of b (or report -1).
//
// In any valid final order, every value v paired with b[i] at position i must
// meet its reverse partner a[n + 1 - i] = b[i] and b[n + 1 - i] = a[i]. So
// the pairing map a_to_b = b o a^{-1} must be an involution (a_to_b(a_to_b(v))
// == v) with at most one fixed point (the middle position when n is odd).
// Placing each paired couple (i, partner) at mirrored slots (lo, hi) and any
// fixed point at the center gives the target index permutation `target: [I]I`;
// each cycle [c0, c1, ..., c_{k-1}] of `target` is realized in k - 1 swaps
// (c0, c_j).

use std.seq.{compose, inverse, perm_cycles}

@!multitest
let n: 2..=200_000 = read()
domain I = 1..=n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()

let inv_a = a.inverse()
let a_to_b = compose(inv_a, b)
let fixed = [v for v in V if a_to_b[v] == v].len
if fixed > 1 or [v for v in V if a_to_b[a_to_b[v]] != v].len > 0:
    print(-1)
    return

var target: [I]I = I.first
var (lo, hi) = (1, n)
for i in I:
    let partner = inv_a[b[i]]
    if i == partner:
        target[i] = I(n / 2 + 1)
    elif i < partner:
        target[i] = I(lo)
        target[partner] = I(hi)
        lo += 1
        hi -= 1

let ops = [(cyc[0], cyc[j]) for cyc in perm_cycles(target) for j in 1..cyc.len]
print(ops.len)
for (u, v) in ops: print(u, v)
