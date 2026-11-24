// Source: NOIP 2010 Senior P1 (Machine Translation), Luogu P1540 (no longer on DMOJ)
// limits: 1000 ms, 128 MB
// Problem: a memory holds M <= 100 words, empty at first. N <= 1000 words (integers <= 1000)
//   are translated in order; a word not in memory is looked up in the dictionary and stored,
//   evicting the word that entered memory earliest when it is full. Print the lookup count.
// Solution: simulate the FIFO cache: a list of stored words with a head index for the oldest,
//   plus a set for membership. A miss pushes the word and, when more than M are live, drops
//   the one at head. O(N log M).
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
