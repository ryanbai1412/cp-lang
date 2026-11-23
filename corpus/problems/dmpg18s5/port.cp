// Source: DMPG 2018 S5 Mimi and Division, https://dmoj.ca/problem/dmpg18s5
// Problem: N, Q <= 2e5 and an array A; queries "1 l r x" print how many of A[l..=r] are divisible
//   by x, "2 u v" sets A[u] = v; values and x are at most 2e5.
// Solution: sqrt decomposition: blocks of sqrt(N) keep, for every d, how many of their values d
//   divides, updated through the divisor lists of all values up to the max (a harmonic sieve). A
//   query scans its partial blocks and adds the counts at x of the whole ones; an update moves
//   the old value's divisors out and the new one's in. O(V log V + (N + Q) (sqrt N + D)), D the
//   most divisors, with N / sqrt(N) * V counters.
// original: corpus/submissions/3476383.cpp

let n: 1..=200_000 = read()
let q: 1..=200_000 = read()
var a: [0..n]Int = read()

var q_type: [0..q]Int = 0
var q_a: [0..q]Int = 0
var q_b: [0..q]Int = 0
var q_c: [0..q]Int = 0
var max_v = 1
for i in 0..n:
    max_v max= a[i]
for i in 0..q:
    let t: 1..=2 = read()
    q_type[i] = t
    if t == 1:
        let l: 1..=200_000 = read()
        let r: 1..=200_000 = read()
        let x: 1..=200_000 = read()
        q_a[i] = l - 1
        q_b[i] = r - 1
        q_c[i] = x
        max_v max= x
    else:
        let idx: 1..=200_000 = read()
        let x: 1..=200_000 = read()
        q_a[i] = idx - 1
        q_b[i] = x
        max_v max= x

var divs: [0..=max_v]List<Int> = []
for d in 1..=max_v:
    var v = d
    while v <= max_v:
        divs[v].push(d)
        v += d

let bsize = n.isqrt
let nb = (n - 1) / bsize + 1
var block_cnt: [0..nb, 0..=max_v]Int = 0

for i in 0..n:
    let b = i / bsize
    let val = a[i]
    for j in 0..divs[val].len:
        block_cnt[b, divs[val][j]] += 1

for qi in 0..q:
    if q_type[qi] == 1:
        let li = q_a[qi]
        let ri = q_b[qi]
        let x = q_c[qi]
        let bl = li / bsize
        let br = ri / bsize
        var ans = 0
        if bl == br:
            for i in li..=ri:
                if a[i] % x == 0:
                    ans += 1
        else:
            let end_l = bsize * (bl + 1)
            for i in li..end_l:
                if a[i] % x == 0:
                    ans += 1
            for i in br * bsize..=ri:
                if a[i] % x == 0:
                    ans += 1
            for b in bl + 1..br:
                ans += block_cnt[b, x]
        print(ans)
    else:
        let i = q_a[qi]
        let x = q_b[qi]
        let b = i / bsize
        let old_v = a[i]
        for j in 0..divs[old_v].len:
            block_cnt[b, divs[old_v][j]] -= 1
        for j in 0..divs[x].len:
            block_cnt[b, divs[x][j]] += 1
        a[i] = x
