// AAC 4 P5 (today/ runnable subset) — original: corpus/submissions/4140767.cpp

let n: 1..=500_000 = read()
let m: 0..=500_000 = read()
let x: 1..=500_000 = read()
let q: 0..=500_000 = read()
domain V = 1..=n

var dsu = Dsu<V>()
for _ in 0..q:
    let u: V = read()
    let v: V = read()
    dsu.unite(u, v)

// Negated MinHeap for MaxHeap<(size, root)>
var pq = MinHeap<(Int, Int)>()
for v in V:
    if dsu.find(v) == v:
        pq.push((-dsu.size(v), -v.idx))

var ok = true
if pq.len < x:
    ok = false

if ok:
    while pq.len > x:
        let p1 = pq.pop()!
        let p2 = pq.pop()!
        let u = V(-p1.1 + 1)
        let v = V(-p2.1 + 1)
        dsu.unite(u, v)
        let r = dsu.find(u)
        pq.push((-dsu.size(r), -r.idx))

    var comp_id: [V]Int = -1
    var cid = 0
    while pq.len > 0:
        let p = pq.pop()!
        let r = V(-p.1 + 1)
        comp_id[r] = cid
        cid += 1

    var comps: [0..x]List<V> = []
    for v in V:
        let r = dsu.find(v)
        comps[comp_id[r]].push(v)

    var eu: List<V> = []
    var ev: List<V> = []
    for i in 0..x:
        let csz = comps[i].len
        if csz > 1:
            eu.push(comps[i][csz - 1])
            ev.push(comps[i][0])
            for j in 0..csz - 1:
                eu.push(comps[i][j])
                ev.push(comps[i][j + 1])

    if eu.len > m:
        ok = false
    else:
        for i in 0..x:
            if eu.len == m:
                break
            let csz = comps[i].len
            for a in 0..csz:
                if eu.len == m:
                    break
                for b in 0..csz:
                    if eu.len == m:
                        break
                    if a != b and (a + 1) % csz != b:
                        eu.push(comps[i][a])
                        ev.push(comps[i][b])

        for i in 0..x:
            if eu.len == m:
                break
            for j in i + 1..x:
                if eu.len == m:
                    break
                for a in 0..comps[i].len:
                    if eu.len == m:
                        break
                    for b in 0..comps[j].len:
                        if eu.len == m:
                            break
                        eu.push(comps[i][a])
                        ev.push(comps[j][b])

        if eu.len != m:
            ok = false
        else:
            for i in 0..m:
                print(eu[i], ev[i])

if not ok:
    print(-1)
