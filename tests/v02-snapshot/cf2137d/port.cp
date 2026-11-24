// Source: Codeforces 2137D (Replace with Occurrences),
//   https://codeforces.com/contest/2137/problem/D
// Problem: given b of length n <= 2e5 (1 <= b_i <= n), build a with 1 <= a_i <= n where the
//   value a_i occurs exactly b_i times in a, or print -1. Multitest, sum n <= 2e5.
// Solution: positions with b_i = x split into groups of x sharing one label, so a exists iff
//   the count of each x is a multiple of x. Bucket positions by b, cut each bucket into chunks
//   of x, and give each chunk a fresh label (numbered by first position). O(n log n).

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
