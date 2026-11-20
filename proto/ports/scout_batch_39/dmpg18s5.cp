// DMPG '18 S5 — original: corpus/submissions/3476383.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

const MAXV = 200_000

let (n, q): (1..=200_000, 1..=200_000) = read()
var a: [0..n]Int = read()

// FRICTION 39.10: sieve(n) and factor(x) return shapes are unspecified and no divisors(x) exists
var divs: [0..=MAXV]List<Int> = [0..=MAXV] _ => []
for d in 1..=MAXV:
    for v in (d..=MAXV).step(d):
        divs[v].push(d)

let bsize = max(1, isqrt(n))
let nb = (n - 1) / bsize + 1
var block_cnt: [0..nb, 0..=MAXV]Int = 0

for i in 0..n:
    let b = i / bsize
    for d in divs[a[i]]:
        block_cnt[b, d] += 1

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let (l, r, x): (1..=n, 1..=n, 1..=MAXV) = read()
        let (li, ri) = (l - 1, r - 1)
        let (bl, br) = (li / bsize, ri / bsize)
        if bl == br:
            print(count(a[i] % x == 0 for i in li..=ri))
        else:
            var ans = count(a[i] % x == 0 for i in li..bsize * (bl + 1))
            ans += count(a[i] % x == 0 for i in br * bsize..=ri)
            ans += sum(block_cnt[b, x] for b in bl + 1..br)
            print(ans)
    else:
        let (idx, x): (1..=n, 1..=MAXV) = read()
        let i = idx - 1
        let b = i / bsize
        for d in divs[a[i]]:
            block_cnt[b, d] -= 1
        for d in divs[x]:
            block_cnt[b, d] += 1
        a[i] = x
