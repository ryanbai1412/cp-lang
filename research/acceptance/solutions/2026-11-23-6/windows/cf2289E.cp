@!multitest
let n: 2..=2e5 = read()
let d: 1..=n = read()
let ch: [0..n - 1](Int, Int) = read()
var dep: [0..2 * n]Int = 0
for k in (n + 1..2 * n).rev:
    let (x, y) = ch[k - n - 1]
    dep[x] = dep[k] + 1
    dep[y] = dep[k] + 1
var hs = [MaxHeap<Int>() for _ in 0..2 * n]
var id = [i for i in 0..2 * n]
var top: [0..2 * n]Int = 0
var ans: [0..=n]Int = 0
for i in 1..=n:
    hs[i].push(i)
    top[i] = i
for k in n + 1..2 * n:
    let (x, y) = ch[k - n - 1]
    for (h, thr) in [(id[x], top[y] + d - 1), (id[y], top[x] + d - 1)]:
        while (hs[h].peek ?? 0) > thr:
            let v = hs[h].pop()!
            ans[v] = dep[v] - dep[k]
    var (a, b) = (id[x], id[y])
    if hs[a].len < hs[b].len: (a, b) = (b, a)
    while let v = hs[b].pop():
        hs[a].push(v)
    id[k] = a
    top[k] = hs[a].peek!
while let v = hs[id[2 * n - 1]].pop():
    ans[v] = dep[v]
print([ans[i] for i in 1..=n])

@alt(program)
fn slow():
    var w: [0..2 * n]List<Int> = [0..2 * n] _ => []
    var par = [0 for _ in 0..2 * n]
    for i in 1..=n:
        w[i] = [i]
    for k in n + 1..2 * n:
        let (x, y) = ch[k - n - 1]
        par[x] = k
        par[y] = k
        var res: List<Int> = []
        for a in w[x]:
            if w[y].any(b => b > a - d): res.push(a)
        for b in w[y]:
            if w[x].any(a => a > b - d): res.push(b)
        w[k] = res
    var out: List<Int> = []
    for i in 1..=n:
        var cnt = 0
        var v = i
        while v != 2 * n - 1 and w[v].contains(i):
            cnt += 1
            v = par[v]
        out.push(cnt)
    print(out)

@gen
fn make_case(size: Int):
    print(1)
    let n = rand(2, max(2, size))
    print(n, rand(1, n))
    var pool = [i for i in 1..=n]
    for k in n + 1..2 * n:
        shuffle(mut pool)
        let x = pool.pop()!
        let y = pool.pop()!
        print(x, y)
        pool.push(k)
