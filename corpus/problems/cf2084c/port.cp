// Source: Codeforces 2084C, https://codeforces.com/contest/2084/problem/C
// limits: 2000 ms, 250 MB
// Problem: permutations a and b of 1..n, n <= 2e5. In at most n operations, each swapping a[i]
//   with a[j] and b[i] with b[j] together, make a[i] = b[n+1-i] for every i; print the swaps,
//   or -1 if impossible. Multitest, sum of n <= 2e5.
// Solution: column i = (a[i], b[i]) must end opposite the column j with a[j] = b[i], so
//   partner(i) = (position of b[i] in a) must be an involution with at most one fixed point,
//   which goes to the middle; otherwise -1. Send the k-th couple to slots k and n+1-k, giving
//   a target permutation; each of its cycles takes len - 1 swaps with its first element. O(n).
// Bounds: n: 2..=200_000; sum(n): 2..=200_000; a, b: 1..=n

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

var target: [I]I = [I] i => i
var k = 0
for i in I if i < inv_a[b[i]]:
    target[i] = I.nth(k)
    target[inv_a[b[i]]] = I.nth(n - 1 - k)
    k += 1
if let mid = I.find(i => inv_a[b[i]] == i):
    target[mid] = I(n / 2 + 1)

var ops: List<(I, I)> = []
ops.reserve(n - 1)
for cyc in perm_cycles(target):
    for j in 1..cyc.len: ops.push((cyc[0], cyc[j]))
print(ops.len)
for (u, v) in ops: print(u, v)
