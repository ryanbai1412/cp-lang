// CF 1795C Tea Tasting
// in: t; per test n, a_1..a_n (tea), b_1..b_n (tasters). out: total each taster drinks, one line.
@!multitest
let n = read<1..=2e5>()
domain V = 1..=n
let a = read<[V]1..=1e9>()
let b = read<[V]1..=1e9>()
let pre = b.scan(+)

var full = Diff<V, Int>()          // how many teas each taster drinks a full b[j] of
var extra: [V]Int = 0              // the leftover a tea leaves to its last taster
for i in V:
    let cap = pre[i.l] + a[i]
    // tea i reaches the tasters before the first cut where the running sum passes cap
    let stop = pre.first_gt(cap) else:
        full.add(i.., 1)
        continue
    let last = stop.prev!
    full.add(i.l..last, 1)
    extra[stop.l!] += cap - pre[last]
let cups = full.finish()
print([V] j => cups[j] * b[j] + extra[j])
