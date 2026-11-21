type Z = Mod<998_244_353>
let comb = Comb<Z.MOD>()

@!multitest
let n: Int = read()
var diff: [0..=n + 1]Z = 0

fn add_range(lo0: Int, hi0: Int, v: Z):
    if lo0 > n or hi0 < -n: return
    let lo = (max(lo0, -n) + n) / 2
    let hi = (min(hi0, n) + n) / 2
    diff[lo] += v
    diff[hi + 1] -= v

fn add_point(x: Int, v: Z): add_range(x, x, v)

add_range(-n, n, 1)
for delta in (2 + n % 2..=n + 2).step(2):
    let x_max = delta - 2
    let cnt = x_max + 1
    let z = if delta == n + 2: 1 else: 2
    for w in (1..=n / delta).step(2):
        add_point(w * delta, -(z * 2 * cnt))
        add_range((w + 1) * delta - x_max, (w + 1) * delta + x_max, z * 2)

for delta in (3 + n % 2..=n + 1).step(2):
    let x_max = delta - 3
    let cnt = x_max + 1
    for w in (1..=(n + 1) / delta).step(2):
        add_point(w * delta - 1, 2 * cnt)
        add_point(w * delta + 1, 2 * cnt)
        add_range((w + 1) * delta - x_max, (w + 1) * delta + x_max, -4)

let counts = scan(diff, +)
print(sum(comb.choose(n, i) * counts[i] for i in 0..=n))
