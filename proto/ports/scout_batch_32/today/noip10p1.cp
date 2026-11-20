// Runnable port (proto/cp.py lib-run) of statement-first noip10p1.cp
// original: corpus/submissions/5445654.cpp
let (m, n): (Int, Int) = read()
var cached = Set<Int>()
var fifo: List<Int> = []
var head = 0
var misses = 0
for _ in 0..n:
    let w: Int = read()
    if w in cached: continue
    misses += 1
    fifo.push(w)
    cached.add(w)
    if fifo.len - head > m:
        cached.remove(fifo[head])
        head += 1
print(misses)
