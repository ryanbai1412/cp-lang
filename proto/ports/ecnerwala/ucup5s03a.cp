// Universal Cup 5 Stage 3 (Gorkovskaya) A: the answer is [x^n] of a quotient of
// two series whose coefficients are exponential convolutions of sequence counts.
use lib.math.poly.{Z, convolve, inv_trunc}

const MOD = 998244353
let (n, k): (Int, Int) = read()
let comb = Comb<MOD>()
let h = n / 2

// sequences with `a` strict and `b` weak comparisons
fn ways(
    a: Int,
    b: Int,
) -> Z = if a > k: Z(0) else: comb.fact(k + b + 1) * comb.inv_fact(a + b + 1) * comb.inv_fact(k - a)

// i! * [x^i] of (sum ways(i + da, i + db) x^i / i!) * e^x
fn egf(da: Int, db: Int) -> List<Z>:
    let c = convolve(
        [ways(i + da, i + db) * comb.inv_fact(i) for i in 0..=h],
        [comb.inv_fact(i) for i in 0..=h],
    )
    [c[i] * comb.fact(i) for i in 0..=h]

let (neg_st, neg_all, pos_st, pos_all) = (egf(0, 0), egf(1, 0), egf(0, 1), egf(1, 1))
var st = [Z(0) for _ in 0..n + 5]
var all = [Z(0) for _ in 0..n + 5]
st[0] = Z(1)
all[0] = Z(1)
all[1] -= Z(k + 1)
for i in 0..=h:
    st[2 * i + 1] -= neg_st[i]
    all[2 * i + 2] += neg_all[i]
    st[2 * i + 2] += pos_st[i]
    all[2 * i + 3] -= pos_all[i]
let res = convolve(st, inv_trunc(all, n + 1))
print(res[n] + Z(n % 2))
