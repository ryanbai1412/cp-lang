// nccc8s3: keep exactly one copy of each of 1..=k so what is left is smallest in dictionary order.
// A monotone stack: a new number pushes out every larger one on top that occurs again later.
let (n, k) = read<(1..=200_000, 1..=200_000)>()
domain V = 1..=k
let a = read<[n]V>()
var last: [V]Int = 0
for (i, x) in a.items:
    last[x] = i
var kept: List<V> = []
var chosen: [V]Bool = false
for (i, x) in a.items:
    if chosen[x]:
        continue
    while let top = kept.last, top > x and last[top] > i:
        chosen[top] = false
        kept.pop()
    kept.push(x)
    chosen[x] = true
print(kept)
