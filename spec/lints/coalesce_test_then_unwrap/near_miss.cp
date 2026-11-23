let prices = [7, 2, 4]
let best = prices.find(x => x > 5)
let other = prices.find(x => x < 3)
// The unwrap names a different value: nothing is said twice.
if (best ?? 0) > 4:
    let b = other!
    print(b)
// The unwrap is not the first thing the branch does.
if (best ?? 0) > 4:
    print(0)
    let b = best!
    print(b)
// No unwrap: the stand-in is the answer for an empty list.
if (best ?? 0) > 4:
    print(1)
print(2)
