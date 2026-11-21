// Matched SortedSet/SortedBag.range(r) benchmark:
// 100k inserts + 100k range .first/.last/.len queries + 20k range iterations.
var bag = SortedBag<Int>()
var rng = 123456789
fn next_rng() -> Int:
    rng = (rng * 1103515245 + 12345) & 2147483647
    rng

for i in 0..100000:
    let _ = bag.add(next_rng() % 50000)

var acc = 0
for _ in 0..100000:
    let a = next_rng() % 50000
    let b = a + (next_rng() % 200)
    acc += bag.range(a..).first ?? 0
    acc += bag.range(..b).last ?? 0
    acc += bag.range(a..b).len

for _ in 0..20000:
    let a = next_rng() % 50000
    for x in bag.range(a..a + 12):
        acc += x

print(acc)
