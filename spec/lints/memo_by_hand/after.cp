let (n, k): (1..=60, 1..=60) = read()
@memo(map) fn ways(i: Int, left: Int) -> Int:
    if i == n:
        return if left == 0: 1 else: 0
    var r = ways(i + 1, left)
    if left > 0:
        r += ways(i + 1, left - 1)
    return r
print(ways(0, k))
