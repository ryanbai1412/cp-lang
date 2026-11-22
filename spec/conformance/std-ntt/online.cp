// expect: ok, output "YES YES YES\nYES YES YES\nYES YES YES YES\nYES YES\nYES"
use std.ntt.{RelaxedConvolve, SemiRelaxedConvolve, convolve_all, subset_exp, subset_log}
type Z = Mod<17>
var rc = RelaxedConvolve<17>()
var rd = RelaxedConvolve<17>()
rd.push(Z(1), Z(3))
var re = RelaxedConvolve<17>()
print(yes(rc.push(Z(0), Z(0)) == Z(0)), yes(rd.push(Z(2), Z(4)) == Z(10)), yes(re.push(Z(1), Z(3)) == Z(3)))
var sc = SemiRelaxedConvolve<17>([Z(1), Z(2)])
let e: List<Z> = []
var sd = SemiRelaxedConvolve<17>(e)
var se = SemiRelaxedConvolve<17>([Z(1), Z(2)])
se.push(Z(1))
se.push(Z(2))
print(yes(sc.push(Z(1)) == Z(1)), yes(sd.push(Z(5)) == Z(0)), yes(se.push(Z(5)) == Z(9)))
let ee: List<List<Z>> = []
print(yes(convolve_all([[Z(1), Z(1)], [Z(1), Z(1)]]) == [Z(1), Z(2), Z(1)]), yes(convolve_all(ee) == [Z(1)]), yes(convolve_all([[Z(1), Z(1)], e]) == []), yes(subset_exp([Z(0), Z(1), Z(1), Z(1)]) == [Z(1), Z(1), Z(1), Z(2)]))
print(yes(subset_log([Z(1), Z(1), Z(1), Z(2)]) == [Z(0), Z(1), Z(1), Z(1)]), yes(subset_log([Z(1), Z(0)]) == [Z(0), Z(0)]))
type W = Mod<998244353>
var seed = 7
fn nxt(s: Int) -> Int = (s * 1103515245 + 12345) % 2147483648
var a = List<W>()
var b = List<W>()
var r = RelaxedConvolve<998244353>()
var bfix = List<W>()
for i in 0..300:
    seed = nxt(seed)
    bfix.push(W(seed))
var s = SemiRelaxedConvolve<998244353>(bfix)
var ok = true
for i in 0..300:
    seed = nxt(seed)
    a.push(W(seed))
    seed = nxt(seed)
    b.push(W(seed))
    let got = r.push(a[i], b[i])
    let got2 = s.push(a[i])
    var want = W(0)
    var want2 = W(0)
    for j in 0..=i:
        want = want + a[j] * b[i - j]
        want2 = want2 + a[j] * bfix[i - j]
    if got != want or got2 != want2: ok = false
var parts = List<List<W>>()
var prod = [W(1)]
for t in 0..9:
    var q = List<W>()
    for u in 0..40:
        seed = nxt(seed)
        q.push(W(seed))
    var np = List.filled(prod.len + q.len - 1, W(0))
    for x in 0..prod.len:
        for y in 0..q.len: np[x + y] = np[x + y] + prod[x] * q[y]
    prod = np
    parts.push(q)
var sub = List<W>()
for m in 0..32:
    seed = nxt(seed)
    sub.push(if m == 0: W(0) else: W(seed))
print(yes(ok and convolve_all(parts) == prod and subset_log(subset_exp(sub)) == sub))
