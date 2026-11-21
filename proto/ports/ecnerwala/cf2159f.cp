fn query(l: Int, t: Int) -> Int:
    print("?", l, t + 1)
    flush()
    read::<Int>() - 1

@!multitest
let (n, m): (Int, Int) = read()
let a: [0..n, 0..n]Int = read()
var loc: [0..n * n]Int = 0
for i in 0..n:
    for j in 0..n:
        loc[a[i, j] - 1] = (if i == 0: i - j else: i + j) + n - 1

var segs = MinHeap<(Int, Int, Int, Int)>()
for l in 1..=n:
    var v0 = max(a[0, j] - 1 for j in 0..l)
    var t0 = 0
    while t0 < 2 * n - 2:
        segs.push((v0, l, t0, t0 + 1))
        let t1 = min(t0 + l, 2 * n - 2)
        let v1 = query(l, t1)
        var (mi, ma) = (t0, t1)
        while ma - mi > 1:
            let md = (mi + ma) / 2
            let vmd = query(l, md)
            if loc[vmd] < t0 + n:
                segs.push((vmd, l, md, mi))
                mi = md
            else:
                segs.push((vmd, l, md, ma))
                ma = md
        (t0, v0) = (t1, v1)
    segs.push((v0, l, t0, t0 + 1))

var ans: List<Int> = []
for _ in 0..m:
    let (v, l, t, t_en) = segs.pop()!
    ans.push(v + 1)
    let nt = t + (t_en - t).sign
    if nt != t_en: segs.push((query(l, nt), l, nt, t_en))
print("!", ans)
flush()
