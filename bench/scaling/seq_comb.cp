// fixed: k
// n: 4096
use std.seq.{lcs}
let n: Int = read()
let ys = [i for i in 0..n]
let m = n / 16
let few = ys.take(32)
let la = [i for i in 0..n / 32]
let lb = [i * 3 % (n / 32) for i in 0..n / 32]
var s = 0
for r in 0..4:
    for (a, b) in ys.take(m).pairs(): // timed: seq.pairs
        s += a ^ b // timed: seq.pairs
    for (a, b) in (0..m).index_pairs(): // timed: seq.index_pairs
        s += a ^ b // timed: seq.index_pairs
    for c in few.permutations(k: 3): // timed: seq.permutations
        s += c.len // timed: seq.permutations
for r in 0..64:
    for c in ys.take(n / 64).combinations(2): // timed: seq.combinations
        s += c.len // timed: seq.combinations
for r in 0..64:
    s += lcs(la, lb).len // timed: seq.lcs
for r in 0..4:
    for sq in ys.take(m).subsequences(): // timed: seq.subsequences
        if sq.items.len >= 2: // timed: seq.subsequences
            sq.prune() // timed: seq.SubseqStep.prune
print(s)
