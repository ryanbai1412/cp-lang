// 3.375e6 Counter.add into a Counter<Int, Int> reserved for them (noi01p5's meet in the middle),
// then as many lookups, half of them misses: a miss walks its whole probe run.
let n = 3_375_000
var c: Counter<Int, Int> = Counter()
c.reserve(n)
var x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) % 2147483647
    c.add(x - 1_000_000_000, 1)
var hits = 0
x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) % 2147483647
    hits += c[if i % 2 == 0: x - 1_000_000_000 else: x + 2_000_000_000]
print(c.len, hits)
