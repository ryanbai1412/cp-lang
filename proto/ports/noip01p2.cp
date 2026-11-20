// cp: 0.1 1f59ac0+dirty
// NOIP 2001 Splitting a Number: the ways to write n as a sum of exactly k
// positive parts, order ignored. Either some part is 1 (drop it), or every
// part is at least 2 (take 1 off each).
let n: 1..=200 = read()
let k: 1..=6 = read()

@memo
fn ways(total: 0..=200, parts: 0..=6) -> Int:
    if parts == 0: return if total == 0: 1 else: 0
    if total < parts: return 0
    ways(total - 1, parts - 1) + ways(total - parts, parts)
print(ways(n, k))
