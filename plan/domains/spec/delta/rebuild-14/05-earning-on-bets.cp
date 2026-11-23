// Codeforces 1979C Earning on Bets
// in: t; per case n, then n multipliers k. out: per case n bets, or -1
@!multitest
let n = read<1..=50>()
let k = read<[n]2..=20>()
let l = k.fold(lcm, init: 1)
let bets = [l / x for x in k]
if bets.sum() < l: print(bets) else: print(-1)
