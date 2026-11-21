const MOD = 676767677

@!multitest
let (n, m): (Int, Int) = read()
let comb = Comb<Mod<MOD>>(n + m + 1)
let ways = a => comb.choose(n - a + m, m - 1)
var ans = Mod<MOD>(0)
var (l_big, l_small) = (n, n - 1)
var (cnt_big, cnt_small) = (1, 0)
var cost = 1
while l_big > 0:
    if cost == 1:
        ans += Mod<MOD>(cnt_big + cnt_small) * ways(1)
    else:
        if cnt_small == 0:
            l_small = l_big
            cnt_small = 1
            cnt_big -= 1
        let c = Mod<MOD>(cost)
        ans += c * Mod<MOD>(cnt_big) * (ways(1) - ways((l_big - 1) / 2 + 2))
        ans += c * Mod<MOD>(cnt_big - 1) * (ways(l_big + 2) - ways(l_big / 2 + 2))
        if l_small > 0:
            ans += c * Mod<MOD>(cnt_small) * (ways(1) - ways(l_small / 2 + 2))
            ans += c * Mod<MOD>(cnt_small - 1) * (ways(l_small + 2) - ways((l_small - 1) / 2 + 2))
    if l_small == 0: break
    let nl_big = l_big / 2
    var (nc_big, nc_small) = (0, 0)
    for (len, cnt) in [(l_big, cnt_big), (l_small, cnt_small)]:
        for d in 0..=1:
            if (len - d) / 2 == nl_big: nc_big += cnt else: nc_small += cnt
    (l_big, l_small) = (nl_big, nl_big - 1)
    (cnt_big, cnt_small) = (nc_big, nc_small)
    cost += 1
print(ans)
