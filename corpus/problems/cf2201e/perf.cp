// Tuned: one convolution instead of two, as the reference does. A side's cell is 0 for `?`,
// 1 for `a` and `base` for `b`, so each cyclic product counts aa + ab·base + bb·base² and
// one NTT pass gives both the agreeing pairs and the known pairs.
use std.math.{factor}

type Z = Mod<998_244_353>

@!multitest
let n: 2..=400_000 = read()
let t: Str = read()
let m = n / 2
let base = m + 1
let pow2 = [0..=m] i => Z(2) ** i

fn fold(from: Int, len: Int) -> ?List<Int>:
    var s = List.filled(len, 0)
    for i in 0..m if t[from + i] != '?':
        let v = if t[from + i] == 'a': 1 else: base
        if s[i % len] != 0 and s[i % len] != v: return None
        s[i % len] = v
    s

fn ways(len: Int) -> Z:
    let x = fold(0, len) ?? return 0
    let y = fold(m, len) ?? return 0
    let pairs = convolve(x, y.reversed(), cyclic: true)
    let fixed = x.count(v => v != 0) + y.count(v => v != 0)
    sum(pow2[len - fixed + v % base + v / base / base] for v in pairs if v / base % base == 0)

print(sum(ways(m / f) * prod(Z(1 - r) for (r, _) in factor(f)) for f in divisors(m)))
