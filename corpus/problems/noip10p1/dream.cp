// NOIP 2010 Machine Translation: a memory of m words, evicted first-in first-out. Count how
// many words must be looked up.
let (m, n) = read (Int, Int)
var memory: Deque<Int> = []
var lookups = 0
for w in read [n]Int:
    if w in memory: continue
    lookups += 1
    memory.push_back(w)
    if memory.len > m: memory.pop_front()
print(lookups)
