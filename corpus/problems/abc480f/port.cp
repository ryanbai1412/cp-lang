// Source: AtCoder ABC 480 F (Cake Cutting), https://atcoder.jp/contests/abc480/tasks/abc480_f
// Problem: N <= 20 cakes of sizes A_i <= 1e9, M <= 100 people, sum S divisible by M. A cut splits
//   one piece in two (any real sizes). Print the fewest cuts after which every person can get
//   exactly S / M.
// Solution: lay the cakes in some order and cut at every multiple of T = S / M: that costs M
//   minus the number of prefixes whose sum is a multiple of T, and is optimal. dp[s] is the most
//   such prefixes over orders of the cakes in s (rem[s] = sum mod T); answer M - dp[full].
//   O(2^N N). The @brute partitions into groups with sums divisible by T over submasks, O(3^N).

let (n, m): (1..=20, 1..=100) = read()
domain I = 0..n
let a: [I]Int = read()

let total = sum(a[i] for i in I)
let target = total / m

var rem: [Subset<I>]Int = 0
for s in Subset<I>:
    if let i = s.first:
        rem[s] = (rem[s.without(i)] + a[i]) % target

var dp: [Subset<I>]Int = 0
for s in Subset<I>:
    for i in Subset<I>.full - s:
        let ns = s.with(i)
        let add = Int(rem[ns] == 0)
        dp[ns] max= dp[s] + add

print(m - dp[Subset<I>.full])

@alt(program)
fn slow():
    let tgt = sum(a[i] for i in I) / m
    var ssum: [Subset<I>]Int = 0
    for s in Subset<I>:
        if let i = s.first:
            ssum[s] = ssum[s.without(i)] + a[i]
    var best: [Subset<I>]Int = -1
    best[Subset<I>.empty] = 0
    for s in Subset<I>:
        if s == Subset<I>.empty or ssum[s] % tgt != 0:
            continue
        for sub in s.subsets:
            if sub != Subset<I>.empty and ssum[sub] % tgt == 0:
                let rest = s - sub
                if best[rest] >= 0:
                    best[s] max= best[rest] + 1
    print(m - best[Subset<I>.full])
