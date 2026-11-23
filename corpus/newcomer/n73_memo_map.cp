let (n, k): (1..=60, 1..=60) = read()
var memo: Map<(Int, Int), Int> = Map()  // expect: memo_by_hand
fn ways(i: Int, left: Int) -> Int:
    if i == n:
        return if left == 0: 1 else: 0
    if (i, left) in memo:
        return memo[(i, left)]
    var r = ways(i + 1, left)
    if left > 0:
        r += ways(i + 1, left - 1)
    memo[(i, left)] = r
    return r
print(ways(0, k))
