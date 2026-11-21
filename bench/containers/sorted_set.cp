// Matched SortedSet bench vs __gnu_pbds::tree:
// 100k inserts + 50k erases + 100k rank (order_of_key) + 100k nth (find_by_order).
var s = SortedSet<Int>()
var rng: U64 = U64(246813579)
var keys: List<Int> = []
keys.reserve(100000)
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let x = Int((rng >> 33) % U64(10000000))
    let _ = s.add(x)
    keys.push(x)
var acc = 0
for i in 0..50000:
    if s.remove(keys[i * 2]): acc += 1
let sz = s.len
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let q = Int((rng >> 33) % U64(10000000))
    acc += s.rank(q)
for _ in 0..100000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let idx = Int((rng >> 33) % U64(sz))
    if let v = s.nth(idx): acc += v
print(acc + s.len)
