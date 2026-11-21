use std.hashmap.{FlatMap}

// Matched FlatMap bench: 300k inserts/updates + 1M lookups.
var fm = FlatMap<Int>.with_capacity(300000, 0)
var s: U64 = U64(314159265)
for i in 0..300000:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let k = Int((s >> 33) % U64(200000))
    fm.set(k, i)
var ans = 0
for _ in 0..1000000:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let k = Int((s >> 33) % U64(400000))
    ans += fm.get(k) ?? 0
print(ans)
