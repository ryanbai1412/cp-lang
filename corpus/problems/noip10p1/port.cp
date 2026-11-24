// Source: NOIP 2010 Senior P1 (Machine Translation), Luogu P1540 (no longer on DMOJ)
// limits: 1000 ms, 128 MB
// Problem: a memory holds M <= 100 words, empty at first. N <= 1000 words (integers <= 1000)
//   are translated in order; a word not in memory is looked up in the dictionary and stored,
//   evicting the word that entered memory earliest when it is full. Print the lookup count.
// Solution: simulate the FIFO cache: a list of stored words with a head index for the oldest,
// Solution: simulate the FIFO cache as a Deque of at most M words: a miss pushes the word
//   and drops the oldest when more than M are held. O(N M).
// original: corpus/submissions/5445654.cpp
// Bounds: m: 1..=100; n: 1..=1000; w: 0..=1000
let (m, n): (1..=100, 1..=1000) = read()
var memory: Deque<Int> = []
var lookups = 0
for w in read<[n]0..=1000>():
    if memory.contains(w): continue
    lookups += 1
    memory.push(w)
    if memory.len > m: memory.pop_front()
print(lookups)
