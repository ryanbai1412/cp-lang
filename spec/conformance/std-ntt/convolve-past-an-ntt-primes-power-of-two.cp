// expect: ok, output "YES YES YES"
// `convolve` over an NTT prime whose `P - 1` has 2^20 but not the product's power of two equals
// the O(n m) loop: 7_340_033 = 7 * 2^20 + 1 takes a product of 2^20 + 99 terms in halves.
type Z = Mod<7_340_033>
let a: List<Z> = [Z(7_340_032 - i * 7919) for i in 0..(1 << 19) + 50]
let b: List<Z> = [Z(i * i * 104_729 + 5) for i in 0..(1 << 19) + 50]
let c = convolve(a, b)
fn at(a: List<Z>, b: List<Z>, k: Int) -> Z:
    var s = Z(0)
    for i in max(0, k - b.len + 1)..=min(k, a.len - 1): s += a[i] * b[k - i]
    s
let short = [Z(i + 3) for i in 0..70]
let d = convolve(short, a)
print(yes(c.len == a.len + b.len - 1), yes(all([c[k] == at(a, b, k) for k in [0, 1, 524_337, 1_048_576, c.len - 1]])), yes(all([d[k] == at(short, a, k) for k in [0, 69, 300_000, d.len - 1]])))
