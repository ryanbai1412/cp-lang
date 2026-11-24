// expect: ok, output "ok"
// input: "1 300\n"
// model: floor_quotients
// model: submasks
// model: bit_subsets
// model: dirichlet_convolve
// model: dirichlet_inv
// model: DirichletSum
// model: sum_phi
// model: sum_mobius
// model: sum_multiplicative
// std.math's divisor-sum machinery and bit enumerators against direct loops: floor quotient
// blocks against one division per i, submask and k-subset walks against a scan of every mask,
// Dirichlet products against the divisor double loop, and Du / Min_25 prefix sums against
// tables built by trial division. Reads `seed n`; prints the first op that differs.
use std.math.{floor_quotients, submasks, bit_subsets, dirichlet_convolve, dirichlet_inv}
use std.math.{DirichletSum, sum_phi, sum_mobius, sum_multiplicative}
use std.rand.{Rng}

const N = 3000

fn ones(x: Int) -> Int:
    var (c, y) = (0, x)
    while y > 0:
        c += y & 1
        y >>= 1
    c

fn quotient_blocks(n: Int, from: Int) -> List<(Int, Int, Int)>:
    var out: List<(Int, Int, Int)> = []
    for i in from..=n:
        let q = n / i
        if out.len > 0 and out.last!.0 == q: out[out.len - 1] = (q, out.last!.1, i + 1)
        else: out.push((q, i, i + 1))
    out

fn conv(a: List<Int>, b: List<Int>, n: Int) -> List<Int>:
    var c = List.filled(n + 1, 0)
    for k in 1..=n:
        for d in 1..=k:
            if k % d == 0: c[k] += a[d] * b[k / d]
    c

// f(k) for f multiplicative with f(p^e) = pp(p, e, p^e), by trial division.
fn mult_value(k: Int, pp: (Int, Int, Int) -> Int) -> Int:
    var (x, r, p) = (k, 1, 2)
    while p * p <= x:
        if x % p == 0:
            var (e, pe) = (0, 1)
            while x % p == 0:
                x /= p
                e += 1
                pe *= p
            r *= pp(p, e, pe)
        p += 1
    if x > 1: r *= pp(x, 1, x)
    r

fn is_p(x: Int) -> Bool = x >= 2 and all(x % d != 0 for d in 2..x if d * d <= x)

var phi = List.filled(N + 1, 0)
var mu = List.filled(N + 1, 0)
for k in 1..=N:
    phi[k] = mult_value(k, (p, e, pe) => pe / p * (p - 1))
    mu[k] = mult_value(k, (p, e, pe) => if e == 1: -1 else: 0)

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 8)
    if op == 0:
        let m = rng.int(0, 200)
        let from = rng.int(1, 30)
        let got = [(t.q, t.lo, t.hi) for t in floor_quotients(m, from:)]
        if got != quotient_blocks(m, from): bad = "op {k}: floor_quotients({m}, from: {from})"
    elif op == 1:
        let mask = rng.int(0, 1023)
        let want = [s for s in (0..=mask).rev if s & mask == s]
        if submasks(mask).collect() != want: bad = "op {k}: submasks({mask})"
    elif op == 2:
        let b = rng.int(0, 10)
        let c = rng.int(0, 12)
        let want = [s for s in 0..1 << b if ones(s) == c]
        if bit_subsets(b, c).collect() != want: bad = "op {k}: bit_subsets({b}, {c})"
    elif op == 3:
        let m = rng.int(0, 60)
        let a = [rng.int(-5, 5) for _ in 0..m + 1 + rng.int(0, 3)]
        let b = [rng.int(-5, 5) for _ in 0..m + 1 + rng.int(0, 3)]
        if dirichlet_convolve(a, b, m) != conv(a, b, m): bad = "op {k}: dirichlet_convolve({a}, {b}, {m})"
    elif op == 4:
        let m = rng.int(0, 60)
        var a = [rng.int(-5, 5) for _ in 0..m + 1]
        if m >= 1: a[1] = if rng.int(0, 1) == 0: 1 else: -1
        let b = dirichlet_inv(a, m)
        var unit = List.filled(m + 1, 0)
        if m >= 1: unit[1] = 1
        if b.len != m + 1 or b[0] != 0 or conv(a, b, m) != unit: bad = "op {k}: dirichlet_inv({a}, {m})"
    elif op == 5:
        let m = rng.int(0, 150)
        let t = max(m, 1)
        let f = [0] + [rng.int(-5, 5) for _ in 1..=t]
        var g = [0] + [rng.int(-3, 3) for _ in 1..=t]
        g[1] = if rng.int(0, 1) == 0: 1 else: -1
        let h = conv(f, g, t)
        var (gs, hs, fs) = (List.filled(t + 1, 0), List.filled(t + 1, 0), List.filled(t + 1, 0))
        for i in 1..=t:
            gs[i] = gs[i - 1] + g[i]
            hs[i] = hs[i - 1] + h[i]
            fs[i] = fs[i - 1] + f[i]
        let small = [f[i] for i in 0..rng.int(0, m + 1)]
        var ds = DirichletSum<Int>(m, small, x => gs[x], x => hs[x])
        for _ in 0..5:
            let q = rng.int(0, m)
            if bad == "" and ds.sum(q) != fs[q]: bad = "op {k}: DirichletSum({f}, {g}, small {small.len}).sum({q})"
    elif op == 6:
        let m = rng.int(-3, N)
        let want = sum(phi[i] for i in 1..=m)
        if sum_phi(m) != want: bad = "op {k}: sum_phi({m})"
    elif op == 7:
        let m = rng.int(-3, N)
        let want = sum(mu[i] for i in 1..=m)
        if sum_mobius(m) != want: bad = "op {k}: sum_mobius({m})"
    else:
        let m = rng.int(0, 600)
        let (c1, c2, c3) = (rng.int(-3, 3), rng.int(-3, 3), rng.int(-3, 3))
        let pp = (p: Int, e: Int, pe: Int) => c1 * (p % 7) + c2 * e + c3
        var vs: List<Int> = []
        for i in (1..=m).rev:
            if vs.is_empty or vs.last! != m / i: vs.push(m / i)
        let ps = [sum(pp(p, 1, p) for p in 2..=v if is_p(p)) for v in vs]
        let want = sum(mult_value(i, pp) for i in 1..=m)
        if sum_multiplicative(m, pp, ps) != want: bad = "op {k}: sum_multiplicative({m}, {c1}, {c2}, {c3})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
