// Codeforces 2089 B2 (Canteen, Hard Version): minimize rounds until `a` becomes
// all zero, after reducing `sum(a)` by at most `k`.
//
// In each round, `a[i]` cancels with `b[i]` and leftover `a[i]` shifts right
// cyclically to `i + 1 (mod n)`. Since `sum(a) <= sum(b)`, cutting the circle
// right after a minimum prefix sum of `a - b` guarantees no surplus crosses
// the cut. Walking once around from that cut, each `b[idx]` cancels greedily
// with the most recent unmatched `a` deposit (LIFO stack), recording how many
// units finish at distance `d = step + 1 - deposit.at`. Deleting a unit drops
// its match, so spending budget `k` on the longest distances leaves the largest
// `d` whose suffix `d..=n` exceeds `k` (or 0 if all units are deleted).

use std.seq.rotated_left

struct Deposit:
    amount: Int
    at: Int

@!multitest
let n: 1..=200_000 = read()
let k: 0..=1_000_000_000_000_000 = read()
domain I = 0..n
let a: [I]Int = read()
let b: [I]Int = read()

let pref = prefix([I] i => a[i] - b[i])
let cut = I.argmin(i => pref.prefix(i.r))!
let order = [i for i in I].rotated_left((cut.ord + 1) % n)

var stk: List<Deposit> = []
var cnt_dist: [0..=n]Int = 0
for (step, idx) in order.items:
    stk.push(Deposit(a[idx], step))
    var rem = b[idx]
    while rem > 0, let top = stk.pop():
        let take = min(top.amount, rem)
        cnt_dist[step + 1 - top.at] += take
        if top.amount > take: stk.push(Deposit(top.amount - take, top.at))
        rem -= take

let suff = prefix(cnt_dist)
let ans = (1..=n).rev.find(d => suff.sum(d..=n) > k) ?? 0
print(ans)
