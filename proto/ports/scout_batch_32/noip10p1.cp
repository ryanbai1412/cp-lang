// NOIP '10 P1 - Machine Translation (DMOJ: noip10p1)
// Ideal v2 port. original: corpus/submissions/5445654.cpp
//
// A dictionary cache holds the m most recently looked-up words, evicting the
// oldest first. Count lookups that miss the cache.
let (m, n): (1..=100, 1..=1000) = read()
let words: [0..n](0..=1000) = read()

var cached: [0..=1000]Bool = false
var fifo: Deque<Int> = []
var misses = 0
for w in words if not cached[w]:
    misses += 1
    cached[w] = true
    fifo.push(w)
    if fifo.len > m: cached[fifo.pop_front()!] = false
print(misses)
