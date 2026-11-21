type Z = Mod<998244353>

@!multitest
let n: Int = read()
var fact: [0..=n]Z = 1
for i in 1..=n: fact[i] = fact[i - 1] * i
var ifact: [0..=n]Z = 1
ifact[n] = fact[n].inv()
for i in (1..=n).rev(): ifact[i - 1] = ifact[i] * i

var pref: [0..(n + 2)]Z = 0
fn add_range(lo0: Int, hi0: Int, v: Z):
    if lo0 > n or hi0 < -n: return
    let lo = (max(lo0, -n) + n) / 2
    let hi = (min(hi0, n) + n) / 2
    pref[lo] += v
    pref[hi + 1] -= v

add_range(-n, n, 1)
var delta = 2 + (n & 1)
while delta <= n + 2:
    let x_max = delta - 2
    let cnt = x_max + 1
    let z = if delta == n + 2: 1 else: 2
    var w = 1
    while w * delta <= n:
        add_range(w * delta, w * delta, -(z * 2 * cnt))
        add_range((w + 1) * delta - x_max, (w + 1) * delta + x_max, z * 2)
        w += 2
    delta += 2

delta = 3 + (n & 1)
while delta <= n + 1:
    let x_max = delta - 3
    let cnt = x_max + 1
    var w = 1
    while w * delta - 1 <= n:
        add_range(w * delta - 1, w * delta - 1, 2 * cnt)
        add_range(w * delta + 1, w * delta + 1, 2 * cnt)
        add_range((w + 1) * delta - x_max, (w + 1) * delta + x_max, -4)
        w += 2
    delta += 2

for i in 0..=n: pref[i + 1] += pref[i]
var ans: Z = 0
for i in 0..=n:
    ans += ifact[i] * ifact[n - i] * pref[i]
print(ans * fact[n])
