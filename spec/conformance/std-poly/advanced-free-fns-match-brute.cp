// expect: ok, output "ok\n2 3 5\n9 16\n2 3 5\n0 1 1 2"
use std.poly.{bostan_mori, transposed_mul, pow_mod_poly, chirp_z, sample_points_shift, power_sums, stirling1_col, stirling2_col, stirling1_row, stirling2_row, inv, eval, lagrange_at}
type Z = Mod<998_244_353>
fn mulmod(a: List<Z>, b: List<Z>, m: List<Z>) -> List<Z>:
    let d = m.len - 1
    var c = List.filled(a.len + b.len, Z(0))
    for i in 0..a.len:
        for j in 0..b.len: c[i + j] += a[i] * b[j]
    for i in (d..c.len).rev:
        let f = c[i] / m[d]
        for j in 0..=d: c[i - d + j] -= f * m[j]
    [c[i] for i in 0..d]
for n in [1, 5, 40, 90]:
    let p = [Z((i * 13 + 7) % 31) for i in 0..n]
    let q = [Z(1)] + [Z((i * 5 + 2) % 17) for i in 1..n + 3]
    for k in [0, 1, 7, 150]:
        let iq = inv(q, k + 1)
        var want = Z(0)
        for i in 0..=k:
            if i < p.len: want += p[i] * iq[k - i]
        assert bostan_mori(p, q, k) == want
    let b = [Z((i * 3 + 1) % 11) for i in 0..(n + 1) / 2]
    let tm = transposed_mul(p, b)
    assert tm.len == p.len - b.len + 1
    for i in 0..tm.len:
        var s = Z(0)
        for j in 0..b.len: s += p[i + j] * b[j]
        assert tm[i] == s
    let m = [Z((i * 7 + 3) % 19) for i in 0..min(n, 20)] + [Z(2)]
    var acc = [Z(1)] + List.filled(m.len - 2, Z(0))
    let base = mulmod(p, [Z(1)], m)
    for k in 0..12:
        assert pow_mod_poly(p, k, m) == acc
        acc = mulmod(acc, base, m)
    for r in [Z(0), Z(1), Z(3), Z(998_244_352)]:
        let cz = chirp_z(p, r, n + 5)
        var x = Z(1)
        for k in 0..n + 5:
            assert cz[k] == eval(p, x)
            x = x * r
    for c in [Z(0), Z(2), Z(n), Z(1000), Z(998_244_350)]:
        let sp = sample_points_shift(p, c, 70)
        for k in 0..70: assert sp[k] == lagrange_at(p, c + Z(k))
    let ps = power_sums(p, 80)
    for k in 0..=80:
        var s = Z(0)
        for x in p:
            var t = Z(1)
            for _ in 0..k: t = t * x
            s += t
        assert ps[k] == s
for nn in [0, 1, 6, 75]:
    for k in [0, 1, 2, 5, 80]:
        let c1: List<Z> = stirling1_col(k, nn)
        let c2: List<Z> = stirling2_col(k, nn)
        assert c1.len == nn + 1 and c2.len == nn + 1
        for i in 0..=nn:
            let r1: List<Z> = stirling1_row(i)
            let r2: List<Z> = stirling2_row(i)
            assert c1[i] == (if k <= i: r1[k] else: Z(0))
            assert c2[i] == (if k <= i: r2[k] else: Z(0))
print("ok")
let a: List<Z> = [1, 1]
print(chirp_z(a, Z(2), 3))
let ys: List<Z> = [0, 1, 4]
print(sample_points_shift(ys, Z(3), 2))
let xs: List<Z> = [1, 2]
print(power_sums(xs, 2))
let s1: List<Z> = stirling1_col(1, 3)
print(s1)
