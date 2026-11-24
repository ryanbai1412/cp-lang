// NOIP 2010 Machine Translation: a memory of m words, evicted first-in first-out. Count how
// many words must be looked up.
let (m, n): (Int, Int) = read()
var memory: Deque<Int> = []
var lookups = 0
for w in read<[n]Int>():
    if memory.each().any(x => x == w): continue
    lookups += 1
    memory.push(w)
    if memory.len > m: memory.pop_front()
print(lookups)
