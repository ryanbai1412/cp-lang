let n: 1..=45 = read()

@memo
fn ways(k: 0..=45) -> Int:
    if k <= 1:
        return 1
    ways(k - 1) + ways(k - 2)

print(ways(n))
