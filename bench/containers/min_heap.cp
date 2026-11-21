// Matched MinHeap bench vs std::priority_queue:
// 500k pushes + 500k interleaved push/pop + drain remaining.
var h = MinHeap<Int>()
var rng: U64 = U64(314159265)
for _ in 0..500000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    h.push(Int((rng >> 33) % U64(1000000000)))
var acc = 0
for _ in 0..500000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    h.push(Int((rng >> 33) % U64(1000000000)))
    if let v = h.pop(): acc += v
while let v = h.pop():
    acc ^= v
print(acc)
