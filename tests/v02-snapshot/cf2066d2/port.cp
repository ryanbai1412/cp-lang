// Source: Codeforces 2066 D2 (Club of Young Aircraft Builders, hard version),
//   https://codeforces.com/contest/2066/problem/D2
// Problem: n <= 100 floors launch m planes in turn (c <= m <= n c); the resident of floor i sees
//   launches from floors <= i and stops once they have seen c, and at the end every resident
//   has seen >= c. Given the launch floors with gaps (0), count the ways to fill the gaps,
//   modulo 1e9 + 7. Multitest, sum of m <= 1e4.
// Solution: decide floors 1..n - 1 bottom up; floor n takes what is left. With `used` launches
//   on lower floors, floor f stops after c more, so all its launches lie in the first used + c
//   (which must cover its last fixed launch). DP over used: choosing k launches for floor f
//   picks k - fixed[f] of the free slots in that prefix, a binomial. O(n m c).

type M = Mod<1_000_000_007>
let comb = Comb<M>(100)

@!multitest
let (n, c, m): (1..=100, 1..=100, c..=n * c) = read()
domain Floor = 1..=n
domain Time = 1..=m
domain Used = 0..=m - c
let a: [Time]?Floor = read(none: 0)

let open = prefix([Time] t => Int(a[t] == None))
var dp: [Used]M = 0
dp[Used.first!] = 1
var fixed_below = 0

for f in Floor if f != Floor.last:
    let fixed = a.count(x => x == f)
    let min_used = match Time.rev.find(t => a[t] == f):
        Some(t) => Used(max(0, t.ord + 1 - c))
        None => Used.first!
    var next: [Used]M = 0
    for used in min_used..=Used.last! if dp[used] != 0:
        let avail = open.fold(Time.first!..=Time.at(used.ord + c)!) + fixed_below - used.ord
        for k in fixed..=min(c, m - c - used.ord):
            next[used.offset(k)!] += dp[used] * comb.choose(avail, k - fixed)
    dp = next
    fixed_below += fixed

print(dp[Used.last!])
