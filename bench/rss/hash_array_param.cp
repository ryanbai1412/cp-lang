// Four ranged maps held in an array and filled through a `mut` param (cf2053f's per-row maps,
// ioi04p2hard's helpers): each still stores its entries as two 32-bit ints, as a local would.
type Key = 0..=1_000_000_000
type Count = 1..=1_000_000
let n: 1..=1_000_000 = read()
var maps: [4]HashMap<Key, Count> = HashMap()
fn fill(mut t: HashMap<Key, Count>, n: 1..=1_000_000, seed: Int):
    t.reserve(n)
    var x = seed
    for i in 0..n:
        x = (x * 1103515245 + 12345) % 2147483647
        t[x % 1_000_000_001] = i + 1
for r in 0..4: fill(mut maps[r], n, 987654321 + r)
var hits = 0
for r in 0..4:
    var x = 987654321 + r
    for i in 0..n:
        x = (x * 1103515245 + 12345) % 2147483647
        hits += maps[r][x % 1_000_000_001]
print(maps[0].len + maps[1].len + maps[2].len + maps[3].len, hits)
