fn query(l: Int, t: Int) -> Int:
    print("?", l, t + 1)
    flush()
    let res: Int = read()
    res - 1

@!multitest
let (n, m): (Int, Int) = read()
let a: [0..(n * n)]Int = read()
var loc: [0..(n * n)]Int = 0
for i in 0..n:
    for j in 0..n:
        let v = a[n * i + j] - 1
        loc[v] = (if i == 0: i - j else: i + j) + (n - 1)

var segs = MinHeap<(Int, Int, Int, Int)>()
for l in 1..=n:
    let o_hi = n
    var v0 = a[0] - 1
    for j in 1..l: v0 max= a[j] - 1
    var t0 = 0
    while t0 < 2 * n - 2:
        segs.push((v0, l, t0, t0 + 1))
        let t1 = min(t0 + l, 2 * n - 2)
        let v1 = query(l, t1)
        var mi = t0
        var ma = t1
        while ma - mi > 1:
            let md = (mi + ma) / 2
            let vmd = query(l, md)
            if loc[vmd] < t0 + o_hi:
                segs.push((vmd, l, md, mi))
                mi = md
            else:
                segs.push((vmd, l, md, ma))
                ma = md
        t0 = t1
        v0 = v1
    segs.push((v0, l, t0, t0 + 1))

var ans: List<Int> = []
for _ in 0..m:
    let (v, l, t, t_en) = segs.pop()!
    ans.push(v + 1)
    let nt = t + (if t_en > t: 1 else: -1)
    if nt != t_en:
        segs.push((query(l, nt), l, nt, t_en))
print("!", ans)
flush()
