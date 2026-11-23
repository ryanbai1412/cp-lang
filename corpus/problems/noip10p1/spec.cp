// noip10p1 Machine Translation: a cache keeps the m most recently added words, evicting the
// oldest. Count the lookups that miss. The deque keeps arrival order; the bitset answers
// membership in O(1), since `in` on a Deque would be a hidden O(m) scan.
domain Word = 0..=1_000
let (m, n) = read<(1..=100, 1..=1_000)>()
var cache: Deque<Word> = Deque()
var cached = Bitset<Word>()
var misses = 0
for w in read<[n]Word>():
    if w in cached: continue
    misses += 1
    cache.push(w)
    cached.add(w)
    if cache.len > m:
        cached.remove(cache.pop_front()!)
print(misses)
