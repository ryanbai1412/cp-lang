// expect: ok, output "ok\n1 6 17\n2 3\n1 2 1\n0 1 998244352 2\n998244352 1"
use std.poly.{SubproductTree, compose_inv, compose, gcd_poly, div_mod, eval}
type Z = Mod<998_244_353>
fn trim(a: List<Z>) -> List<Z>:
    var b = a.clone()
    while not b.is_empty and b[b.len - 1] == Z(0): b.pop()
    b
fn brute_gcd(a: List<Z>, b: List<Z>) -> List<Z>:
    var (x, y) = (trim(a), trim(b))
    while not y.is_empty: (x, y) = (y, trim(div_mod(x, y).r))
    if x.is_empty: return x
    let l = x[x.len - 1].inv()
    [c * l for c in x]
fn mul(a: List<Z>, b: List<Z>) -> List<Z>:
    var c = List.filled(a.len + b.len - 1, Z(0))
    for i in 0..a.len:
        for j in 0..b.len: c[i + j] += a[i] * b[j]
    c
for m in [1, 3, 33, 70]:
    let xs = [Z(i * i * 7 + 3 * i + 1) for i in 0..m]
    let t = SubproductTree(xs)
    let f = [Z((i * 17 + 5) % 41) for i in 0..m + 9]
    let ev = t.eval(f)
    for i in 0..m: assert ev[i] == eval(f, xs[i])
    let c = [Z((i * 3 + 2) % 13) for i in 0..m]
    for tt in [t, SubproductTree(xs, keep: m)]:
        let et = tt.eval_transposed(c)
        for k in 0..m:
            var s = Z(0)
            for i in 0..m:
                var p = Z(1)
                for _ in 0..k: p = p * xs[i]
                s += c[i] * p
            assert et[k] == s
    let ys = [Z((i * 29 + 11) % 37) for i in 0..m]
    let poly = t.interpolate(ys)
    assert poly.len == m
    for i in 0..m: assert eval(poly, xs[i]) == ys[i]
for n in [1, 2, 5, 40, 130]:
    let f = [Z(0)] + [Z((i * 5 + 3) % 23 + 1) for i in 1..n + 2]
    let g = compose_inv(f, n)
    assert g.len == n
    let back = compose(f, g, n)
    for i in 0..n: assert back[i] == (if i == 1: Z(1) else: Z(0))
for (da, db, dg) in [(0, 0, 0), (5, 3, 2), (40, 30, 10), (150, 120, 70), (90, 90, 0), (200, 1, 0)]:
    let g = [Z((i * 31 + 7) % 53 + 1) for i in 0..dg + 1]
    let a = mul(g, [Z((i * 11 + 1) % 19 + 1) for i in 0..da + 1])
    let b = mul(g, [Z((i * 13 + 4) % 29 + 1) for i in 0..db + 1])
    assert gcd_poly(a, b) == brute_gcd(a, b)
    assert gcd_poly(b, a) == brute_gcd(a, b)
let e: List<Z> = []
assert gcd_poly(e, e) == e
print("ok")
let xs: List<Z> = [0, 1, 2]
print(SubproductTree(xs).eval([Z(1), Z(2), Z(3)]))
let x2: List<Z> = [1, 2]
print(SubproductTree(x2).eval_transposed([Z(1), Z(1)]))
print(SubproductTree(xs).interpolate([Z(1), Z(4), Z(9)]))
let f: List<Z> = [0, 1, 1]
print(compose_inv(f, 4))
let a: List<Z> = [2, -3, 1]
let b: List<Z> = [3, -4, 1]
print(gcd_poly(a, b))
