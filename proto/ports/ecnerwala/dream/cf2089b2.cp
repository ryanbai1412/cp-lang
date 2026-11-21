// Codeforces 2089 B2 (Canteen, Hard Version): minimize rounds until a becomes
// all zero, after reducing sum(a) by at most k.
//
// In each round, a[i] cancels with b[i] and any leftover a[i] shifts right
// cyclically to i + 1 (mod n). Because sum(a) <= sum(b), the total net flow
// sum(a - b) <= 0; cutting the circle right after a minimum prefix sum of
// a - b guarantees no surplus ever crosses the cut. Walking once around from
// that cut, each b[idx] cancels greedily with the most recent unmatched a
// deposit (LIFO stack), recording how many units finish after distance d =
// step + 1 - deposit.at rounds. Deleting a unit from a[pos] simply drops its
// match, so spending the budget k on the longest distances first is optimal.

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
let (_, cut) = [(pref.prefix(i.r), i) for i in I].min()!
let start = (cut.ord + 1) % n

var stk: List<Deposit> = []
var cnt_dist: [0..=n]Int = 0
for step in 0..n:
    let idx = I((start + step) % n)
    stk.push(Deposit(a[idx], step))
    var rem = b[idx]
    while rem > 0 and not stk.is_empty:
        let top = stk.len - 1
        let take = min(stk[top].amount, rem)
        cnt_dist[step + 1 - stk[top].at] += take
        stk[top].amount -= take
        if stk[top].amount == 0: stk.pop()
        rem -= take

var budget = k
for d in (1..=n).rev:
    let spend = min(budget, cnt_dist[d])
    budget -= spend
    if cnt_dist[d] > spend:
        print(d)
        return
print(0)
