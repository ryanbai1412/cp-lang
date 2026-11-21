// Matched Dsu bench: 500k nodes, 500k random unites + 1M same/size queries.
let n = 500000
domain V = 0..n
var d = Dsu<V>()
var s: U64 = U64(135792468)
var acc = 0
for _ in 0..500000:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let u = V(Int((s >> 33) % U64(n)))
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let v = V(Int((s >> 33) % U64(n)))
    if d.unite(u, v): acc += 1
for _ in 0..1000000:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let u = V(Int((s >> 33) % U64(n)))
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let v = V(Int((s >> 33) % U64(n)))
    if d.same(u, v): acc += d.size(u)
print(acc + d.parts)
