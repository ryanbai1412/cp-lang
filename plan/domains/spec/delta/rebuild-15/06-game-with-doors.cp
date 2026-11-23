// CF 2004B Game with Doors
// in: t; per test l r, then L R. out: the fewest doors to lock.
@!multitest
let (l1, r1): (1..=100, 1..=100) = read()
let (l2, r2): (1..=100, 1..=100) = read()
let lo = max(l1, l2)
let hi = min(r1, r2)
if lo > hi:
    print(1)
    return
print(hi - lo + Int(l1 != l2) + Int(r1 != r2))
