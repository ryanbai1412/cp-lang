@!multitest
fn rev_edge(e: List<Int>) -> List<Int>:
    var r = List.filled(9, 0)
    for i in 0..3:
        for j in 0..3:
            r[i * 3 + j] = e[j * 3 + i]
    r
fn merge_parallel(a: List<Int>, b: List<Int>) -> List<Int>:
    var r = List.filled(9, 0)
    for ai in 0..3:
        for bi in 0..(3 - ai):
            for aj in 0..3:
                for bj in 0..(3 - aj):
                    r[(ai + bi) * 3 + (aj + bj)] max= a[ai * 3 + aj] + b[bi * 3 + bj]
    r
fn merge_series(a: List<Int>, v: List<Int>, b: List<Int>) -> List<Int>:
    var r = List.filled(9, 0)
    for ai in 0..3:
        for aj in 0..3:
            for bj in 0..(3 - aj):
                for bk in 0..3:
                    r[ai * 3 + bk] max= a[ai * 3 + aj] + v[2 - (aj + bj)] + b[bj * 3 + bk]
    r
fn merge_leaf(v: List<Int>, e: List<Int>, l: List<Int>) -> List<Int>:
    var r = List.filled(3, 0)
    for a in 0..3:
        for i in 0..(3 - a):
            for j in 0..3:
                r[a + i] max= v[a] + e[i * 3 + j] + l[2 - j]
    r[1] max= r[0]
    r[2] max= r[1]
    r
let (n, m): (Int, Int) = read()
var mp: Map<(Int, Int), List<Int>> = Map()
var adj: List<Set<Int>> = [Set() for _ in 0..n]
var vert_val: List<List<Int>> = [List.filled(3, 0) for _ in 0..n]
let init_e: List<Int> = [0, 0, 0, 0, 1, 1, 0, 1, 1]
for _ in 0..m:
    let (u0, v0): (Int, Int) = read()
    let (u, v) = (min(u0 - 1, v0 - 1), max(u0 - 1, v0 - 1))
    mp[(u, v)] = init_e
    adj[u].insert(v)
    adj[v].insert(u)
var in_q: [0..n]Bool = false
var q: Deque<Int> = Deque()
for i in 0..n:
    if adj[i].len <= 2:
        in_q[i] = true
        q.push(i)
while let cur = q.pop_front():
    if adj[cur].len == 2:
        let u0 = adj[cur].pop_first()!
        let v0 = adj[cur].pop_last()!
        adj[u0].remove(cur)
        adj[v0].remove(cur)
        let (u, v) = (min(u0, v0), max(u0, v0))
        let k1 = (min(u, cur), max(u, cur))
        let raw1 = mp[k1]
        mp.remove(k1)
        let e1 = if u < cur: raw1 else: rev_edge(raw1)
        let k2 = (min(cur, v), max(cur, v))
        let raw2 = mp[k2]
        mp.remove(k2)
        let e2 = if cur < v: raw2 else: rev_edge(raw2)
        let e = merge_series(e1, vert_val[cur], e2)
        if (u, v) in mp:
            mp[(u, v)] = merge_parallel(mp[(u, v)], e)
        else:
            mp[(u, v)] = e
        adj[u].insert(v)
        adj[v].insert(u)
        if not in_q[u] and adj[u].len <= 2:
            in_q[u] = true
            q.push(u)
        if not in_q[v] and adj[v].len <= 2:
            in_q[v] = true
            q.push(v)
    elif adj[cur].len == 1:
        let o = adj[cur].pop_first()!
        adj[o].remove(cur)
        let key = (min(o, cur), max(o, cur))
        let raw = mp[key]
        mp.remove(key)
        let e = if o < cur: raw else: rev_edge(raw)
        vert_val[o] = merge_leaf(vert_val[o], e, vert_val[cur])
        if not in_q[o] and adj[o].len <= 2:
            in_q[o] = true
            q.push(o)
    else:
        print(vert_val[cur][2])
