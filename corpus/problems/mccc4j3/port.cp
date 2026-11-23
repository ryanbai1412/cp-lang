// Runnable port (proto/cp.py lib-run) of mccc4j3.cp
// original: corpus/submissions/4264907.cpp

let n: Int = read()
domain V = 1..=n
let a: [V]Int = read()

var total = 0
for i in V: total += a[i]

var cur = 0
var ans: ?V = None
for i in V:
    cur += a[i]
    if 2 * cur == total:
        ans = i
        break

if let idx = ans: print(idx)
else: print("Andrew is sad.")
