// expect: ok, output "YES YES YES"
// `convolve` over `F64` past 60 terms is a complex FFT: each output is within its documented
// bound, (sum a^2 + sum b^2) * log2(n) * 2^-52, of the O(n m) loop, and integer inputs round back.
fn brute(a: List<F64>, b: List<F64>) -> List<F64>:
    var out = List.filled(a.len + b.len - 1, 0.0)
    for i in 0..a.len:
        for j in 0..b.len: out[i + j] += a[i] * b[j]
    out
let a = [F64((i * 7919) % 2001 - 1000) / 7.0 for i in 0..700]
let b = [F64((i * 104_729) % 1999 - 999) / 3.0 for i in 0..333]
let c = convolve(a, b)
let d = brute(a, b)
var energy = 0.0
for x in a: energy += x * x
for x in b: energy += x * x
let bound = energy * 10.0 * 2.0 ** -52.0
var worst = 0.0
for k in 0..c.len: worst = max(worst, abs(c[k] - d[k]))
let ia = [F64(i % 1000) for i in 0..500]
let ib = [F64((i * 37) % 1000) for i in 0..400]
let ic = convolve(ia, ib)
let id = brute(ia, ib)
print(yes(c.len == d.len), yes(worst <= bound), yes([F64(round(x)) for x in ic] == id))
