@!multitest
let n: 2..=2e5 = read()
let k: 1..=min(n, 1500) = read()
let a: [0..n]1..=1e9 = read()
var cand: [0..n]List<(d: Int, h: Int)> = [0..n] _ => []
for p in 0..n:
    var lst: List<(d: Int, h: Int)> = []
    for h in 1..=min(k + 1, n - 1 - p):
        let diff = a[p + h] - a[p]
        if diff % h == 0: lst.push((d: diff / h, h: h))
    lst.sort()
    for x in lst:
        if cand[p].len == 0 or cand[p][cand[p].len - 1].d != x.d:
            cand[p].push(x)
var vis: [0..n]List<Bool> = [0..n] p => [false for _ in 0..cand[p].len]
fn find(q: Int, d: Int) -> Int:
    let i = cand[q].bsearch(x => x.d >= d)
    if i < cand[q].len and cand[q][i].d == d: i else: -1
var best = 1
for p in 0..n:
    for idx in 0..cand[p].len:
        if vis[p][idx]: continue
        let d = cand[p][idx].d
        var pts: List<Int> = [p]
        var (cur, ci) = (p, idx)
        loop:
            vis[cur][ci] = true
            let nxt = cur + cand[cur][ci].h
            pts.push(nxt)
            let j = find(nxt, d)
            if j < 0: break
            (cur, ci) = (nxt, j)
        var lo = 0
        var off = 0
        for hi in 0..pts.len:
            if hi > 0: off += pts[hi] - pts[hi - 1] - 1
            while off > k:
                off -= pts[lo + 1] - pts[lo] - 1
                lo += 1
            best max= hi - lo + 1
print(min(n, best + k))

@alt(program)
fn slow():
    var ans = 1
    for l in 0..n:
        for r in l..n:
            let len = r - l + 1
            var ok = len <= k + 1
            for i in l..=r:
                for j in i + 1..=r:
                    if (a[j] - a[i]) % (j - i) == 0:
                        let d = (a[j] - a[i]) / (j - i)
                        var on = 0
                        for t in l..=r:
                            if a[t] - a[i] == d * (t - i): on += 1
                        if len - on <= k: ok = true
            if ok: ans max= len
    print(ans)
