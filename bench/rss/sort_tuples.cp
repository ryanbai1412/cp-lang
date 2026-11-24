let m: 1..=2e5 = read()
let given: [0..m](s: 0..=1000, e: 0..=1000) = read()
var events: List<(Int, Bool, Int)> = []
events.reserve(2 * m)
for i in 0..m:
    events.push((-given[i].s, false, i))
    events.push((-given[i].e, true, i))
events.sort()
var h = 0
for (t, arrival, i) in events:
    h = (h * 31 + t + (if arrival: 7 else: 0) + i) % 1_000_000_007
print(h)
