// Matched Map/Set bench vs std::map/std::set:
// 100k inserts + 100k first_ge + 100k range(a..b).first/.last + 20k short range iters.
var m: Map<Int, Int> = Map()
var s: Set<Int> = Set()
var rng: U64 = U64(987654321)
for i in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let k = Int((rng >> 33) % U64(10000000))
    m[k] = i
    s.add(k)
var acc = 0
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let q = Int((rng >> 33) % U64(10000000))
    if let (k, v) = m.first_ge(q): acc += k + v
    if let k = s.first_ge(q): acc += k
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let lo = Int((rng >> 33) % U64(9900000))
    let hi = lo + 50000
    if let (k, v) = m.range(lo..hi).first: acc += k + v
    if let k = s.range(lo..hi).last: acc += k
for _ in 0..20000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let lo = Int((rng >> 33) % U64(9999000))
    let hi = lo + 500
    for (k, v) in m.range(lo..hi): acc += k + v
    for k in s.range(lo..hi): acc += k
print(acc + m.len + s.len)
