// CCO '24 P2 — Pizza Party (today form for proto/cpc)
// original: corpus/submissions/6435481.cpp
let n: 1..=1_000_000 = read()
domain I = 0..n

var arra: [I]Int = 0
var arrb: [I]Int = 0
var cnt_a: [I]Int = 0
var cnt_b: [I]Int = 0
var max_a = 0
var max_b = 0

for i in 0..n:
    let x: Int = read()
    let v = x - 1
    arra[I(i)] = v
    cnt_a[I(v)] += 1
    if v > max_a: max_a = v

for i in 0..n:
    let x: Int = read()
    let v = x - 1
    arrb[I(i)] = v
    cnt_b[I(v)] += 1
    if v > max_b: max_b = v

var same_tally = true
var perm_ok = true
for i in 0..n:
    if cnt_a[I(i)] != cnt_b[I(i)]: same_tally = false
    if cnt_a[I(i)] != 1: perm_ok = false

if not same_tally:
    print(-1)
elif max_a <= 1 and max_b <= 1:
    var rev_eq = true
    for i in 0..n:
        if arra[I(i)] != arrb[I(n - 1 - i)]: rev_eq = false
    if rev_eq:
        print(1)
        var ones: List<Int> = List.filled(n, 1)
        print(ones)
        print(ones)
    else:
        print(2)
        var out_a: List<Int> = []
        var out_b: List<Int> = []
        for i in 0..n:
            out_a.push(arra[I(i)] + 1)
            out_b.push(arrb[I(i)] + 1)
        print(out_a)
        print(out_b)
elif perm_ok:
    var pos_rev: [I]Int = 0
    for i in 0..n:
        pos_rev[I(arra[I(n - 1 - i)])] = i

    var tops: List<Int> = []
    var sid: [I]Int = 0
    for i in 0..n:
        let val = arrb[I(i)]
        let cx = pos_rev[I(val)]
        let lo = (0..tops.len).first_true(mid => tops[mid] < cx) ?? tops.len
        if lo == tops.len:
            tops.push(cx)
        else:
            tops[lo] = cx
        sid[I(val)] = lo + 1

    print(tops.len)
    var out_a: List<Int> = []
    var out_b: List<Int> = []
    for i in 0..n:
        out_a.push(sid[I(arra[I(i)])])
        out_b.push(sid[I(arrb[I(i)])])
    print(out_a)
    print(out_b)
