// Codeforces 1927D: Find the Different Ones!
// I/O: t tests; each n, a_1..a_n, q, then q lines l r; per query print i j in [l, r] with a_i != a_j, or -1 -1.
@!multitest
let n: 2..=2e5 = read()
domain I = 1..=n
let a = read<[I]1..=1e6>()
// diff[i]: the last j < i with a[j] != a[i]
var diff: [I]?I = None
for i in I:
    if let j = i.prev:
        diff[i] = if a[j] != a[i]: j else: diff[j]
let q: 1..=2e5 = read()
for e in read<[q](l: I, r: I)>():
    if let j = diff[e.r], j >= e.l: print(j, e.r)
    else: print(-1, -1)
