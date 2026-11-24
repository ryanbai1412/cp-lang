let (n, k): (1..=2e5, 0..=1e9) = read()
let a: [0..n]0..=1e9 = read()
var seen: Counter<Int, Int> = Counter()
seen[0] += 1
var acc = 0
var count = 0
for x in a:
    acc ^= x
    count += seen.get(acc ^ k)
    seen[acc] += 1
print(count)
