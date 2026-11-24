// Universal Cup 5 Stage 5 F, tuned, after orig.cpp:
// - Power sums from a product tree of the monic leaves x - s, each product taken mod
//   x^m - 1 with m = bit_ceil(degree), so a node of power-of-two degree takes a transform of
//   its own size and not twice it; the wrapped top coefficient is 1 and comes back off
//   position 0. Only the first `deg` terms of p' enter the convolution.
// - orig.cpp's transposed prefix-product pass in place of port.cp's multipoint evaluation and
//   Taylor shift. With F(k) = x (x - 1) ... (x - k + 1), answer[k] = <F(k), p> / k!, p the
//   power sums. Walk the tree of the leaves x - l down, carrying c[j] = sum_m prefix[m] p[m + j]
//   with prefix the product of the leaves left of the node: a left child keeps c's front, a
//   right child takes the middle product of c with its left sibling.
use std.poly.{deriv, inv, transposed_mul}
use std.strings.{suffix_tree_edges}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

// the product of two monic polynomials of degree >= 1
fn mul_monic(a: List<Z>, b: List<Z>) -> List<Z>:
    let d = a.len + b.len - 2
    if d <= 64: return convolve(a, b)
    let m = bit_ceil(d)
    var fa = List.filled(m, Z(0))
    var fb = List.filled(m, Z(0))
    unchecked:
        for i in 0..a.len: fa[i] = a[i]
        for i in 0..b.len: fb[i] = b[i]
    ntt(mut fa, bit_reversed: true)
    ntt(mut fb, bit_reversed: true)
    unchecked:
        for i in 0..m: fa[i] *= fb[i]
    ntt(mut fa, inverse: true, bit_reversed: true)
    if d == m:
        fa[0] -= Z(1)
        fa.push(Z(1))
    else:
        fa.truncate(d + 1)
    fa

fn prod_monic(xs: List<Z>, lo: Int, hi: Int) -> List<Z>:
    if hi - lo == 1: return [-xs[lo], Z(1)]
    let mid = (lo + hi) / 2
    mul_monic(prod_monic(xs, lo, mid), prod_monic(xs, mid, hi))

// sum of x^j over xs for j in 0..=deg
fn power_sums(xs: List<Z>, deg: Int) -> List<Z>:
    var out = List.filled(deg + 1, Z(0))
    out[0] = Z(xs.len)
    if xs.is_empty: return out
    let p = prod_monic(xs, 0, xs.len).rev.to_list()
    var dp = deriv(p)
    if dp.len > deg: dp.truncate(deg)
    let q = convolve(dp, inv(p, deg))
    for k in 1..=deg: out[k] = if k - 1 < q.len: -q[k - 1] else: Z(0)
    out

fn build(mut prods: List<List<Z>>, k: Int, lo: Int, hi: Int):
    if hi - lo == 1:
        prods[k] = [-Z(lo), Z(1)]
        return
    let mid = (lo + hi) / 2
    build(mut prods, 2 * k, lo, mid)
    build(mut prods, 2 * k + 1, mid, hi)
    prods[k] = mul_monic(prods[2 * k], prods[2 * k + 1])

fn down(prods: List<List<Z>>, mut out: List<Z>, k: Int, lo: Int, hi: Int, c: List<Z>):
    if hi - lo == 1:
        out[lo] = c[1] - c[0] * Z(lo)
        return
    let mid = (lo + hi) / 2
    down(prods, mut out, 2 * k, lo, mid, c[..mid - lo + 1].to_list())
    down(prods, mut out, 2 * k + 1, mid, hi, transposed_mul(c, prods[2 * k]))

@!multitest
let n: 1..=2e5 = read()
let s: Str of "abcdefghijklmnopqrstuvwxyz" = read()
let edges = suffix_tree_edges(s)
let bottom = power_sums([Z(e.bottom_size) for e in edges], n)
let top = power_sums([Z(e.top_size) for e in edges if e.top_size > 0], n)
var prods: List<List<Z>> = List.filled(4 * n, [])
build(mut prods, 1, 0, n)
var falling = List.filled(n, Z(0))
down(prods, mut falling, 1, 0, n, [bottom[j] - top[j] for j in 0..=n])
print([falling[i] * comb.inv_fact(i + 1) for i in 0..n])
