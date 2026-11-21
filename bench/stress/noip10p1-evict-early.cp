// NOIP 2010 P1 machine translation (proto/ports/scout_batch_32/noip10p1.cp), with a planted
// off-by-one: a word is evicted once the memory holds `m` words, not more than `m`, so the
// memory holds one word too few. The words are read one per pass of a loop.
let (m, n): (1..=100, 1..=1000) = read()
var words: List<Int> = []
for _ in 0..n:
    let w: 0..=1000 = read()
    words.push(w)

fn misses(m: Int, words: List<Int>) -> Int:
    var cached = Set<Int>()
    var fifo: List<Int> = []
    var head = 0
    var count = 0
    for w in words:
        if w in cached: continue
        count += 1
        fifo.push(w)
        cached.add(w)
        if fifo.len - head >= m:
            cached.remove(fifo[head])
            head += 1
    count

@brute(misses)
fn misses_slow(m: Int, words: List<Int>) -> Int:
    var memory: List<Int> = []
    var count = 0
    for w in words:
        if any(x == w for x in memory): continue
        count += 1
        memory.push(w)
        if memory.len > m: memory = [memory[i] for i in 1..memory.len]
    count

print(misses(m, words))
