// Runnable port (proto/cp.py lib-run) of etsr23r4p1.cp
// original: corpus/submissions/5521420.cpp

let n: Int = read()
domain V = 0..n
var p: [V]V = [V] v => v
for i in 0..n:
    let x: Int = read()
    p[V(i)] = V(x - 1)

var vis: [V]Bool = false
var base: [V]Int = 0
var big: List<List<V>> = []
var bad_odd = false

for v in V:
    if vis[v]: continue
    var cyc: List<V> = []
    var u = v
    while not vis[u]:
        vis[u] = true
        cyc.push(u)
        u = p[u]
    if cyc.len % 2 != 0:
        bad_odd = true
        break
    if cyc.len == 2:
        let a = if cyc[0].to_idx < cyc[1].to_idx: cyc[0] else: cyc[1]
        let b = if cyc[0].to_idx < cyc[1].to_idx: cyc[1] else: cyc[0]
        base[a] = 1
        base[b] = -1
    else:
        big.push(cyc)

if not bad_odd:
    let m = big.len
    var found = false
    for mask in 0..(1 << m):
        var cur: [V]Int = [V] v => base[v]
        for j in 0..m:
            let bit = (mask >> j) & 1
            let cyc = big[j]
            for k in 0..cyc.len:
                cur[cyc[k]] += ((k ^ bit) % 2) * 2 - 1
        var psum = 0
        var ok = true
        for v in V:
            psum += cur[v]
            if psum < 0:
                ok = false
                break
        if ok:
            var out = ""
            for v in V:
                if cur[v] == -1: out += ")"
                else: out += "("
            print(out)
            found = true
            break
