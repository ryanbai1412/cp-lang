// Baltic OI '04 P3 (Sequence): L1 isotonic regression via PAV + skew heap
// (array-backed arena in proto/cpc; MaxHeap small-to-large in v2).
// Runnable on proto/cpc today.
// original: corpus/submissions/5270736.cpp
let n: 1..=1e6 = read()
domain V = 0..n
let a: [V]Int = read()
var x: [V]Int = 0
for i in V: x[i] = a[i] - i.idx

// Array-backed max skew heap: node i starts with value x[i].
var hval: [V]Int = 0
var hlc: [V]?V = none
var hrc: [V]?V = none
for i in V: hval[i] = x[i]

fn merge_heap(a_in: ?V, b_in: ?V) -> ?V:
    if a_in == none: return b_in
    if b_in == none: return a_in
    var u = a_in!
    var v = b_in!
    if hval[u] < hval[v]:
        let tmp = u
        u = v
        v = tmp
    let rc = hrc[u]
    hrc[u] = merge_heap(rc, v)
    let tmp = hlc[u]
    hlc[u] = hrc[u]
    hrc[u] = tmp
    u

var st_l: List<Int> = []
var st_r: List<Int> = []
var st_sz: List<Int> = []
var st_rt: List<V> = []

for i in 0..n:
    var cl = i
    var cr = i + 1
    var csz = 1
    var crt = V(i)
    while st_rt.len > 0:
        let top_rt = st_rt[st_rt.len - 1]
        if hval[top_rt] <= hval[crt]: break
        cl = st_l.pop()!
        st_r.pop()
        csz += st_sz.pop()!
        let prev_rt = st_rt.pop()!
        crt = merge_heap(prev_rt, crt)!
        let need = (cr - cl + 1) / 2
        while csz > need:
            let lc = hlc[crt]
            let rc = hrc[crt]
            crt = merge_heap(lc, rc)!
            csz -= 1
    st_l.push(cl)
    st_r.push(cr)
    st_sz.push(csz)
    st_rt.push(crt)

var cost = 0
for k in 0..st_rt.len:
    let md = hval[st_rt[k]]
    for i in st_l[k]..st_r[k]: cost += abs(x[V(i)] - md)
print(cost)
for k in 0..st_rt.len:
    let md = hval[st_rt[k]]
    for i in st_l[k]..st_r[k]: print(md + i)
