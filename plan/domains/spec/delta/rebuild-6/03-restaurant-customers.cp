// CSES 1619 Restaurant Customers: the most customers present at once.
// Input: n, then n lines "a b" (arrival, leaving; all times distinct). Output: the maximum.
let n = read<1..=2e5>()
let stays = read<[n](a: 1..=1e9, b: 1..=1e9)>()
let events = chain(stays.map(s => (s.a, 1)), stays.map(s => (s.b, -1))).sorted()
var here = 0
var most = 0
for (_, d) in events:
    here += d
    most max= here
print(most)
