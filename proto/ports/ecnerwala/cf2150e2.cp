use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: Int = read()
if n == 1:
    print("!", 1)
    flush()
    return

var p = List(1..2 * n)
rng.shuffle(mut p)

fn ask(v: Int, l: Int, r: Int) -> Int:
    print("?", v + 1, r - l, p[l..r])
    flush()
    read()

var cnt: [0..n]Int = 2
var (l, r) = (0, 2 * n - 1)
let m = (l + r) / 2
var cnt_l: [0..n]Int = 0
var cnt_r: [0..n]Int = 0
for i in 0..n:
    cnt_l[i] = ask(i, l, m)
    if cnt_l[i] == 0:
        cnt_r[i] = 2
    else:
        cnt_r[i] = ask(i, m, r)
        let s = 2 / (cnt_l[i] + cnt_r[i])
        cnt_l[i] *= s
        cnt_r[i] *= s
if sum(cnt_l) > m - l:
    r = m
    cnt = cnt_l
else:
    l = m
    cnt = cnt_r

var ans = 0
'outer: for i in 0..n if cnt[i] == 2:
    var (mi, ma) = (l, r)
    while true:
        if ma - mi == 1:
            ans = i
            break 'outer
        let md = (mi + ma) / 2
        if ask(i, mi, md) == 0: mi = md
        elif ask(i, md, ma) == 0: ma = md
        else: break
print("!", ans + 1)
flush()
