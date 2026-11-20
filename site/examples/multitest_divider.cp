// Precomputed once before any test case runs.
var tri: [0..=20]Int = 0
for i in 1..=20:
    tri[i] = tri[i - 1] + i

@!multitest

let (n, step): (0..=20, 1..=3) = read()
if n == 0:
    print(0)
    return

// Fresh @memo cache per test case (depends on per-case step).
@memo
fn ways(x: 0..=20) -> Int:
    if x == 0:
        return 1
    var total = ways(x - 1)
    if step >= 2 and x >= 2:
        total += ways(x - 2)
    if step == 3 and x >= 3:
        total += ways(x - 3)
    total

print(ways(n), tri[n])
