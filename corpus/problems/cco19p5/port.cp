// Source: CCO 2019 P5 (Marshmallow Molecules), https://dmoj.ca/problem/cco19p5
// Problem: N <= 1e5 marshmallows and M <= 1e5 required skewers (a, b), a < b. The set must be
//   closed: if a < b < c and skewers a-b and a-c exist, so does b-c. Print the fewest skewers
//   in a closed set containing all the required ones.
// Solution: go up from vertex 1 with S = v's higher neighbours; closure joins every pair in S,
//   but joining the lowest f in S to the rest suffices, as f's own turn then joins those
//   among themselves. So count |S|, remove f and merge S into f's set, smaller into larger
//   (swapping the sets first). O((N + M) log^2).
let (n, m): (Int, Int) = read()
domain V = 1..=n
domain Stick = 0..m
let sticks: [Stick](lo: V, hi: V) = read()

var above = [V] v => Set<V>()  // the higher neighbours of v
for (lo, hi) in sticks: above[lo].add(hi)

var total = 0
for v in V:
    total += above[v].len
    if above[v].len == 0: continue
    let f = above[v].min()!
    above[v].remove(f)
    if f != v and above[f].len < above[v].len: swap(mut above[f], mut above[v])
    for x in above[v].copy(): above[f].add(x)
print(total)
