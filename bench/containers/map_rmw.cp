// Matched Map bench vs std::map: 4M `m[k] = (m.get(k) ?? 0) + v` over 100k struct keys, the
// read-modify-write of cf2237i1's merge DP. The store and the read now find the entry once.
struct Key(a: Int, b: Bool)
var m: Map<Key, Int> = Map()
var rng: U64 = U64(987654321)
for i in 0..4000000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let r = Int((rng >> 33) % U64(100000))
    let k = Key(r / 2, r % 2 == 1)
    m[k] = (m.get(k) ?? 0) + i % 1000
var acc = 0
for (k, v) in m.items:
    acc = (acc * 31 + k.a + v) % 1_000_000_007
print(acc + m.len)
