// CF 2014H Robin Hood Archery: the Sheriff (second player) does not lose iff every
// value occurs an even number of times in a[l..=r]; test it with a xor hash of values.
use std.math.{splitmix64}

@!multitest
let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a = read<[V]1..=1e6>()
let pre = ([V] i => splitmix64(U64(a[i]))).scan(^)
repeat q:
    let (l, r): (V, V) = read()
    print(yes(pre[r.r] ^ pre[l.l] == U64(0)))
