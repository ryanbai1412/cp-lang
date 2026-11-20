// Runnable port (proto/cp.py lib-run) of mccc3j5.cp
// original: corpus/submissions/4223449.cpp

let (n, m): (Int, Int) = read()
var eu: List<Int> = []
var ev: List<Int> = []
for _ in 0..m:
    let (u, v): (Int, Int) = read()
    eu.push(u - 1)
    ev.push(v - 1)

var ans = 0
for mask in 0..(1 << n):
    var ok = true
    for i in 0..m:
        if ((mask >> eu[i]) & 1) == 1 and ((mask >> ev[i]) & 1) == 1:
            ok = false
            break
    if ok:
        let sz = popcount(mask)
        if sz > ans: ans = sz

print(ans)
