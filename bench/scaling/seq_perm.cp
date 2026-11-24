// The permutations are local (a reversal, a shift): a random one's walk is
// random access, whose time per element climbs with n until it leaves the
// caches and the TLB (0.6 ns at 2^14 to 2.1 at 2^20 for partial_inverse),
// which is the machine's growth, not the fn's.
use std.seq.{compose, perm_pow, perm_cycles}
let n: Int = read()
domain V = 0..n
var p: [V]V = [V] v => v
var q: [V]V = [V] v => v
for k in 0..n:
    p[V.nth(k)] = V.nth(n - 1 - k)
    q[V.nth(k)] = V.nth((k + 1) % n)
let pp: [V]?V = [V] v => if v == V.nth(0): None else: Some(p[v])
var w = [0] + [n - i for i in 1..n]
var s = 0
for r in 0..2048:
    s += Int(p.inverse()[V.nth(0)] == V.nth(0)) // timed: seq.inverse
    s += Int(compose(q, p)[V.nth(0)] == V.nth(0)) // timed: seq.compose
    s += Int(pp.partial_inverse()[V.nth(1)] == None) // timed: seq.partial_inverse
    s += Int(w.next_permutation()) // timed: seq.next_permutation
    s += Int(w.prev_permutation()) // timed: seq.prev_permutation
for r in 0..256:
    s += Int(perm_pow(q, 3)[V.nth(0)] == V.nth(0)) // timed: seq.perm_pow
    s += perm_cycles(p).len + perm_cycles(q).len // timed: seq.perm_cycles
    for v in q.cycle(from: V.nth(0)): // timed: seq.cycle
        s += Int(v == V.nth(1)) // timed: seq.cycle
print(s)
