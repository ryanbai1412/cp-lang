// noip10p1 Machine Translation: a cache keeps the m most recently added words, evicting the
// oldest. Count the lookups that miss.
let (m, n) = read<(1..=100, 1..=1_000)>()
var cache: Deque<Int> = Deque()
var misses = 0
for w in read<[n]0..=1_000>():
    if w not in cache:
        misses += 1
        cache.push(w)
        if cache.len > m:
            cache.pop_front()
print(misses)
