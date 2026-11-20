// NOIP 2001 Splitting a Number: the ways to write n as a sum of exactly k
// positive parts, order ignored. Either some part is 1 (drop it), or every
// part is at least 2 (take 1 off each).
let (n, k): (1..=200, 1..=6) = read()

@memo
fn ways(total: 0..=n, parts: 0..=k) -> Int =
    if parts == 0: Int(total == 0)
    elif total < parts: 0
    else: ways(total - 1, parts - 1) + ways(total - parts, parts)
print(ways(n, k))
