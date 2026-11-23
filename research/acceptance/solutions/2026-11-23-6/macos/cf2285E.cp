@!multitest
let (n, k): (2..=2e5, 1..=1500) = read()
let a: [0..n]1..=1e9 = read()
var best = min(n, k + 1)
// node = (position j, difference d): j is in the class of d whose previous member is within k + 1
var nd: List<Int> = []     // d
var ncnt: List<Int> = []   // members from the chain head through this node
var ngap: List<Int> = []   // changed elements from the chain head through this node
var nst: List<Int> = []    // window start: a node id, or -(f + 1) for the virtual head before node f
var nnext: List<Int> = []
var off: [0..n]Int = 0
var len: [0..n]Int = 0
for j in 0..n:
    var cand: List<(Int, Int)> = []
    for delta in 1..=min(k + 1, j):
        let diff = a[j] - a[j - delta]
        if diff % delta == 0:
            cand.push((diff / delta, delta))
    cand.sort()
    off[j] = nd.len
    for (d, delta) in cand:
        if nd.len > off[j] and nd[nd.len - 1] == d: continue
        let p = j - delta
        var lo = off[p]
        var hi = off[p] + len[p]
        while lo < hi:
            let mid = (lo + hi) / 2
            if nd[mid] < d: lo = mid + 1 else: hi = mid
        let id = nd.len
        nd.push(d)
        nnext.push(-1)
        if lo < off[p] + len[p] and nd[lo] == d:
            nnext[lo] = id
            ncnt.push(ncnt[lo] + 1)
            ngap.push(ngap[lo] + delta - 1)
            nst.push(nst[lo])
        else:
            ncnt.push(2)
            ngap.push(delta - 1)
            nst.push(-(id + 1))
        var st = nst[id]
        fn gap_at(s: Int) -> Int = if s < 0: 0 else: ngap[s]
        while ngap[id] - gap_at(st) > k:
            st = if st < 0: -st - 1 else: nnext[st]
        nst[id] = st
        let cnt_before = if st < 0: 0 else: ncnt[st] - 1
        best max= min(n, ncnt[id] - cnt_before + k)
    len[j] = nd.len - off[j]
print(best)
