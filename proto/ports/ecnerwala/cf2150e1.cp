use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: Int = read()
var p: List<Int> = [i + 1 for i in 0..(2 * n - 1)]
rng.shuffle(mut p)

fn ask(v: Int, l: Int, r: Int) -> Int:
    print("?", v + 1, r - l, p[l..r])
    flush()
    read()

var cnt: [0..n]Int = 2
var l = 0
var r = 2 * n - 1
while true:
    var cands = 0
    var who = 0
    for i in 0..n:
        if cnt[i] == 2:
            cands += 1
            who = i
    if cands == 1:
        print("!", who + 1)
        flush()
        break
    let m = (l + r) / 2
    var cnt_l: [0..n]Int = 0
    var cnt_r: [0..n]Int = 0
    for i in 0..n:
        if cnt[i] == 2:
            cnt_l[i] = ask(i, l, m)
            if cnt_l[i] == 0:
                cnt_r[i] = 2
            else:
                cnt_r[i] = ask(i, m, r)
                let s = 2 / (cnt_l[i] + cnt_r[i])
                cnt_l[i] *= s
                cnt_r[i] *= s
    for i in 0..n:
        if cnt[i] == 1:
            cnt_r[i] = ask(i, m, r)
            cnt_l[i] = 1 - cnt_r[i]
    let tot_l = sum(cnt_l)
    if tot_l > m - l:
        r = m
        cnt = cnt_l
    else:
        l = m
        cnt = cnt_r
