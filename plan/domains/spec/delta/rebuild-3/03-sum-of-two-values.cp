// CSES 1640 Sum of Two Values: two distinct positions whose values sum to x.
// In: n x, then a1..an. Out: the two positions (1-based), or IMPOSSIBLE.
let n = read<1..=2e5>()
let x = read<1..=1e9>()
domain I = 1..=n
let a = read<[I]1..=1e9>()
var seen: Map<Int, I> = [:]
for i in I:
    if let j = seen.get(x - a[i]):
        print(j, i)
        return
    seen[a[i]] = i
print("IMPOSSIBLE")
