// CF 1902E Collapsing Strings. |C(a, b)| = |a| + |b| - 2 lcp(rev a, b), so the
// answer is 2 n L - 2 * sum over a of the number of (string, prefix) pairs
// matching each prefix of rev a. Every prefix hash of every string goes in one
// table; all hashes share fixed bases so they compare.
let n: 1..=1e6 = read()
let ss = read<[n]Str of "abcdefghijklmnopqrstuvwxyz">()
let bases = (91_138_233, 97_266_353)
var cnt: Map<HashVal, Int> = Map()
var total = 0
for s in ss:
    total += s.len
    let h = RollingHash(s, bases: bases)
    for k in 1..=s.len: cnt[h.get(0..k)] += 1
var common = 0
for s in ss:
    let h = RollingHash(s.rev, bases: bases)
    for k in 1..=s.len:
        let c = cnt.get(h.get(0..k)) ?? 0
        if c == 0: break
        common += c
print(2 * n * total - 2 * common)
