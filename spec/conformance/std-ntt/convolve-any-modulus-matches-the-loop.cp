// expect: ok, output "YES YES YES YES"
// `convolve` over a `Mod<P>` whose `P - 1` lacks the power of two (1e9+7, and a runtime modulus
// past 2^31) takes three NTT primes and Garner's CRT, and equals the O(n m) loop, cyclic too.
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
print(yes(convolve(a, b) == brute(a, b, W(0))), yes(convolve(ra, rb) == brute(ra, rb, R(0))), yes(convolve(ca, cb, cyclic: true) == cyc), yes(convolve(b, a) == convolve(a, b)))
