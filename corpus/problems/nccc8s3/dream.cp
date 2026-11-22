// NCCC 8 S3: the lexicographically smallest subsequence holding each of the k values exactly
// once. Monotone stack: drop a larger value from the top while it appears again later.
let (n, k) = read (Int, Int)
domain K = 1..=k
let a = read [n]K
let last = a.last_index_of()          // [K] last position of each value
var stack: List<K> = []
var held: [K]Bool = false
for (i, x) in a.items:
    if held[x]: continue
    while stack.last is some(y) and y > x and last[y] > i:
        held[stack.pop()!] = false
    stack.push(x)
    held[x] = true
print(stack)
