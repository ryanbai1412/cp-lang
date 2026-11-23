// CSES 1619 Restaurant Customers: the most customers present at once (all times distinct).
// Input: n, then n lines "a b" (arrival, leaving). Output: the maximum count.
let n = read<1..=2e5>()
let guests = read<[n](a: 1..=1e9, b: 1..=1e9)>()

let events = ([(g.a, 1) for g in guests] + [(g.b, -1) for g in guests]).sorted()
var (now, best) = (0, 0)
for (_, d) in events:
    now += d
    best max= now
print(best)
