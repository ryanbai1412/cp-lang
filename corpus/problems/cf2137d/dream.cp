// Codeforces 2137 D — Replace with Occurrences
//
// Every distinct label assigned to positions with `b[i] == x` must appear
// exactly `x` times, so a valid array `a` exists iff for every `x in 1..=n`,
// the count of positions with `b[i] == x` is divisible by `x`. When valid,
// partition each `groups[x]` into chunks of size `x` and assign a fresh label
// to each chunk.

@!multitest
let n: 1..=200_000 = read()
let b: [0..n]1..=n = read()
var groups: [1..=n]List<Int> = []
for (i, x) in b.items: groups[x].push(i)

if not (1..=n).all(x => groups[x].len % x == 0):
    print(-1)
    return

var runs: List<List<Int>> = []
for x in 1..=n:
    for chunk in chunks(groups[x], x): runs.push(chunk)
runs.sort(by: c => c[0])

var a: [0..n]Int = 0
for (id, chunk) in runs.items:
    for i in chunk: a[i] = id + 1
print(a)
