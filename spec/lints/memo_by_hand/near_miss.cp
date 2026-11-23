let (n, k): (1..=60, 1..=60) = read()
var memo: Map<(Int, Int), Int> = Map()
fn ways(i: Int, left: Int) -> Int:
    if (i, left) in memo:
        return 0
    memo[(i, left)] = 1
    return if i == n: 1 else: ways(i + 1, left)
print(ways(0, k))
