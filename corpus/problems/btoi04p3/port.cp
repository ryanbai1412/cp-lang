// Source: Baltic OI 2004 Day 1 P3 (Sequence), https://dmoj.ca/problem/btoi04p3
// limits: 2000 ms, 512 MB
// Problem: given t_1..t_N (N <= 1e6, 0 <= t_i <= 2e9), find a strictly increasing integer
//   z_1 < ... < z_N (|z_i| <= 2e9) minimizing sum |t_i - z_i|; print that minimum, then
//   any optimal z, one per line.
// Solution: with x_i = t_i - i and y_i = z_i - i the order becomes non-decreasing: L1
//   isotonic regression. Pool adjacent violators: keep a stack of blocks, each fitted by
//   its median; merge the new block into the top while the top's median is larger. Each
//   block keeps its smallest ceil(len / 2) values in a max skew heap (top = median), so a
//   merge melds the heaps and pops down to size: O(N log N); z_i = median + i.
// original: corpus/submissions/5270736.cpp
// Bounds: n: 1..=1e6
let n: 1..=1e6 = read()
domain V = 0..n
let a: [V]Int = read()
var x: [V]Int = 0
for i in V: x[i] = a[i] - i.idx

// Array-backed max skew heap: node i starts with value x[i].
var hval: [V]Int = 0
var hlc: [V]?V = None
var hrc: [V]?V = None
for i in V: hval[i] = x[i]

fn merge_heap(a_in: ?V, b_in: ?V) -> ?V:
    if a_in == None: return b_in
    if b_in == None: return a_in
    var u = a_in!
    var v = b_in!
    if hval[u] < hval[v]:
        swap(mut u, mut v)
    let rc = hrc[u]
    hrc[u] = merge_heap(rc, v)
    swap(mut hlc[u], mut hrc[u])
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
        let top_rt = st_rt.last!
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
