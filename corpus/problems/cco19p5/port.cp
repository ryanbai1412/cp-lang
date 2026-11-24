// Source: CCO 2019 P5 (Marshmallow Molecules), https://dmoj.ca/problem/cco19p5
// Problem: N <= 1e5 marshmallows and M <= 1e5 required skewers (a, b), a < b. The set must be
//   closed: if a < b < c and skewers a-b and a-c exist, so does b-c. Print the fewest skewers
//   in a closed set containing all the required ones.
// Solution: go up from vertex 1 with S = v's higher neighbours; closure joins every pair in S,
//   but joining the lowest f in S to the rest suffices, as f's own turn then joins those
//   among themselves. So count |S|, remove f and absorb S into f's SortedSet (smaller into
//   larger). O((N + M) log^2).
let n: 1..=1e5 = read()
let m: 1..=1e5 = read()
domain V = 1..=n
let sticks: [m](lo: V, hi: V) = read()

var above = [V] v => SortedSet<V>()  // the higher neighbours of v
for s in sticks: above[s.lo].add(s.hi)

var total = 0
for v in V:
    total += above[v].len
    if let f = above[v].first:
        above[v].remove(f)
        above[f].absorb(mut above[v])
print(total)
