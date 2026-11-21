// Codeforces 2150 E1 (interactive): randomly permute indices and binary-search
// the half whose remaining element multiplicities exceed its length.
use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: Int = read()
var p = [i for i in 1..2 * n]
rng.shuffle(mut p)

fn ask(v: Int, lo: Int, hi: Int) -> Int:
    print("?", v + 1, hi - lo, p[lo..hi])
    flush()
    read()

var cnt: [0..n]Int = 2
var (l, r) = (0, 2 * n - 1)
while true:
    let alive = [i for i in 0..n if cnt[i] == 2]
    if alive.len == 1:
        print("!", alive[0] + 1)
        flush()
        return

    let m = (l + r) / 2
    var cnt_l: [0..n]Int = 0
    var cnt_r: [0..n]Int = 0
    for i in alive:
        cnt_l[i] = ask(i, l, m)
        if cnt_l[i] == 0:
            cnt_r[i] = 2
        else:
            cnt_r[i] = ask(i, m, r)
            let s = 2 / (cnt_l[i] + cnt_r[i])
            cnt_l[i] *= s
            cnt_r[i] *= s
    for i in 0..n if cnt[i] == 1:
        cnt_r[i] = ask(i, m, r)
        cnt_l[i] = 1 - cnt_r[i]

    if sum(cnt_l) > m - l:
        r = m
        cnt = cnt_l
    else:
        l = m
        cnt = cnt_r
