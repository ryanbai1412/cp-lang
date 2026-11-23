// Universal Cup 5 Stage 5 (Chengdu) F: answer[k] = sum of choose(size, k) over
// suffix-tree edges, bottom end minus top end. Since subtree sizes reach n^2,
// take their power sums via the logarithmic derivative -x p'(x) / p(x) of
// p(x) = prod(1 - s x), then convert power sums to binomial sums via transposed
// multipoint evaluation:
//   answer[k] = [x^k] (power_sums(x) * prod(1 - u x for u in 0..k)) / k!
use std.poly.{compose, inv, prod_linear}
use std.strings.{suffix_tree_edges}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

// sum of s^j over the sizes, for j in 0..=deg (entry 0 is left at zero)
fn power_sums(sizes: List<Z>, deg: Int) -> List<Z>:
    let p = prod_linear(sizes).rev.to_list().resized(deg + 1, Z(0))
    let r = convolve([-Z(i) * p[i] for i in 0..=deg], inv(p, deg + 1))
    r[..deg + 1].to_list()

@!multitest
let n: 1..=2e5 = read()
let s: Str = read()
let edges = suffix_tree_edges(s)
let above = [Z(e.bottom_size) for e in edges]
let below = [Z(e.top_size) for e in edges]

let top = power_sums(above, n)
let bottom = power_sums(below, n)
let sums = [top[j] - bottom[j] for j in 0..=n]
let f = [sums[j] * comb.inv_fact(j) for j in 0..=n]
let ln1p = [Z(0)] + [(if k % 2 == 1: Z(1) else: -Z(1)) * comb.inv(k) for k in 1..=n]
let ans = compose(f, ln1p, n + 1)
print(ans[1..])
