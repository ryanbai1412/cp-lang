// 1954D Colored Balls: sorted ascending, the newest color is the max; value is max(a_max, ceil(S/2)).
type Z = Mod<998244353>
let n: 1..=5000 = read()
let a = read<[n]1..=5000>().sorted()
var cnt: [a.sum() + 1]Z = 0
cnt[0] = 1
var reach = 0
var ans: Z = 0
for x in a:
    for s in 0..=reach:
        ans += cnt[s] * max(x, (s + x).ceil_div(2))
    for s in (0..=reach).rev:
        cnt[s + x] += cnt[s]
    reach += x
print(ans)
