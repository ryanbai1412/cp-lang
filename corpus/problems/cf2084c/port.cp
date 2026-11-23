// Codeforces 2084 C: simultaneously swap indices in two permutations a and b
// until a becomes the reverse of b (or report -1).
//
// In any valid final order, every value v paired with b[i] at position i must
// meet its reverse partner a[n + 1 - i] = b[i] and b[n + 1 - i] = a[i]. So
// the pairing map a_to_b = b o a^{-1} must be an involution (a_to_b(a_to_b(v))
// == v) with at most one fixed point (the middle position when n is odd).
// Zipping the paired couples (i, partner) with mirrored slots from I and I.rev
// (and placing any fixed point at the center) gives the target permutation
// `target: [I]I`; each cycle [c0, c1, ..., c_{k-1}] of `target` is realized in
// k - 1 swaps (c0, c_j).

use std.seq.{compose, perm_cycles}

@!multitest
let n: 2..=200_000 = read()
domain I = 1..=n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()

let inv_a = a.inverse()
let a_to_b = compose(inv_a, b)
if V.count(v => a_to_b[v] == v) > 1 or V.any(v => a_to_b[a_to_b[v]] != v):
    print(-1)
    return

let couples = [(i, inv_a[b[i]]) for i in I if i < inv_a[b[i]]]
var target: [I]I = [I] i => i
for (k, (u, v)) in couples.items:
    target[u] = I.nth(k)
    target[v] = I.nth(n - 1 - k)
if let mid = I.find(i => inv_a[b[i]] == i):
    target[mid] = I(n / 2 + 1)

let ops = [(cyc[0], cyc[j]) for cyc in perm_cycles(target) for j in 1..cyc.len]
print(ops.len)
for (u, v) in ops: print(u, v)
