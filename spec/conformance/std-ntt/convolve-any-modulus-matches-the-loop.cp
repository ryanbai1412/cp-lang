// expect: ok, output "YES YES YES\nYES YES YES YES"
// `convolve` over a `Mod<P>` whose `P - 1` lacks the power of two equals the O(n m) loop, cyclic
// too: 1e9+7 and a runtime 2e9 take the split FFT, a modulus past 2^31 and a product too long
// for the FFT's precision take three NTT primes and Garner's CRT.
type W = Mod<1_000_000_007>
let m = 2_000_000_000
type R = Mod<m>
fn brute<T: Add + Mul>(a: List<T>, b: List<T>, zero: T) -> List<T>:
    var out = List.filled(a.len + b.len - 1, zero)
    for i in 0..a.len:
        for j in 0..b.len: out[i + j] = out[i + j] + a[i] * b[j]
    out
let a: List<W> = [W(1_000_000_006 - i * 7919) for i in 0..300]
let b: List<W> = [W(i * i * 104_729 + 5) for i in 0..211]
let ra: List<R> = [R(1_999_999_999 - i * 15_485_863) for i in 0..257]
let rb: List<R> = [R(i * 32_452_843 + 1) for i in 0..130]
let ca = [W(i + 1) for i in 0..128]
let cb = [W(1_000_000_006 - 3 * i) for i in 0..128]
let lin = brute(ca, cb, W(0))
let cyc = [if k + 128 < lin.len: lin[k] + lin[k + 128] else: lin[k] for k in 0..128]
type Q = Mod<3_000_000_017>
let qa: List<Q> = [Q(3_000_000_016 - i * 999_983) for i in 0..150]
let qb: List<Q> = [Q(i * 7_368_787 + 2) for i in 0..97]
let la: List<R> = [R(1_999_999_999 - i * 15_485_863) for i in 0..17_000]
let lb: List<R> = [R(i * 32_452_843 + 1) for i in 0..16_000]
let lc = convolve(la, lb)
fn at<T: Add + Mul>(a: List<T>, b: List<T>, k: Int, zero: T) -> T:
    var s = zero
    for i in max(0, k - b.len + 1)..=min(k, a.len - 1): s = s + a[i] * b[k - i]
    s
let sa = la.take(8_000)
let sb = lb.take(8_000)
let sc = convolve(sa, sb)
let near = all([sc[k] == at(sa, sb, k, R(0)) for k in [0, 4_321, 7_999, 12_345, 15_998]])
let sampled = all([lc[k] == at(la, lb, k, R(0)) for k in [0, 1, 999, 16_000, 16_999, 25_000, 32_998]])
print(yes(convolve(qa, qb) == brute(qa, qb, Q(0))), yes(lc.len == 32_999 and sampled), yes(sc.len == 15_999 and near))
print(yes(convolve(a, b) == brute(a, b, W(0))), yes(convolve(ra, rb) == brute(ra, rb, R(0))), yes(convolve(ca, cb, cyclic: true) == cyc), yes(convolve(b, a) == convolve(a, b)))
