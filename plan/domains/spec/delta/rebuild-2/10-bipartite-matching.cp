// Bipartite matching: n boys and m girls; row i is "k g1 .. gk", the girls boy i
// accepts. Print the maximum number of couples, then each couple "boy girl".
use std.flow.{max_matching}
let n = read<1..=1e5>()
let m = read<1..=1e5>()
domain Boy = 1..=n
domain Girl = 1..=m
let likes = read<[Boy]List<Girl>>()
let r = max_matching(likes)
print(r.size)
for b in Boy:
    if let g = r.mate[b]: print(b, g)
