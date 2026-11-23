// Source: Codeforces 2089B2 (Canteen, Hard Version),
//   https://codeforces.com/contest/2089/problem/B2
// Problem: a[0..n) and b[0..n), sum a <= sum b. First make exactly k decrements of positive a_i.
//   Then each round both a_i and b_i drop by min(a_i, b_i) and a shifts right cyclically by one.
//   Print the fewest rounds until a is all 0. n <= 2e5, k <= 2e14, sum of n <= 2e5; multitest.
// Solution: cut the circle right after a minimum prefix sum of a - b, so no surplus crosses the
//   cut. Walking once round from it, each b[idx] eats the latest unmatched a deposits (a LIFO
//   stack); a unit eaten d = step + 1 - deposit's step rounds later counts at distance d. The k
//   decrements delete the farthest units, so the answer is the largest d whose suffix count
//   d..=n exceeds k (0 if none). O(n).

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
let cut = ([I] i => pref.prefix(i.r)).argmin()!
let order = I.collect().rotated_left(cut.ord + 1)

var stk: List<Deposit> = []
var cnt_dist: [0..=n]Int = 0
for (step, idx) in order.items:
    stk.push(Deposit(a[idx], step))
    var rem = b[idx]
    while rem > 0 and not stk.is_empty:
        let top = stk.pop()!
        let take = min(top.amount, rem)
        cnt_dist[step + 1 - top.at] += take
        if top.amount > take: stk.push(Deposit(top.amount - take, top.at))
        rem -= take

let suff = prefix(cnt_dist)
let ans = (1..=n).rev.find(d => suff.fold(d..=n) > k) ?? 0
print(ans)
