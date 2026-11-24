// NCCC 8 S3: the lexicographically smallest subsequence holding each of the k values exactly
// once. Monotone stack: drop a larger value from the top while it appears again later.
let (n, k) = read<(1..=200_000, 1..=200_000)>()
domain K = 1..=k
let a = read<[n]K>()
var last: [K]Int = 0                  // the last position of each value
for (i, x) in a.items: last[x] = i
var stack: List<K> = []
var held: [K]Bool = false
for (i, x) in a.items:
    if held[x]: continue
    while stack.last is Some(y) and y > x and last[y] > i:
        held[stack.pop()!] = false
    stack.push(x)
    held[x] = true
print(stack)
