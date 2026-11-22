// Universal Cup 5 Stage 5 (Chengdu) F: answer[k] = sum of choose(size, k) over
// suffix-tree edges, bottom end minus top end. Since subtree sizes reach n^2,
// take their power sums via the logarithmic derivative -x p'(x) / p(x) of
// p(x) = prod(1 - s x), then convert power sums to binomial sums via transposed
// multipoint evaluation:
//   answer[k] = [x^k] (power_sums(x) * prod(1 - u x for u in 0..k)) / k!
use std.ntt.{convolve}
use std.poly.{inv, prod_linear}
use std.strings.{suffix_array, lcp_array}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

struct OpenNode(depth: Int, size: Int)

// seg[node] = prod(1 - roots[i] x for i in lo..hi), cut to `keep` terms
fn product(mut seg: List<List<Z>>, node: Int, lo: Int, hi: Int, roots: List<Z>, keep: Int):
    if hi - lo == 1:
        seg[node] = [Z(1), -roots[lo]]
        return
    let mid = (lo + hi) / 2
    product(mut seg, 2 * node, lo, mid, roots, keep)
    product(mut seg, 2 * node + 1, mid, hi, roots, keep)
    let p = convolve(seg[2 * node], seg[2 * node + 1])
    seg[node] = if p.len > keep: p[..keep] else: p

// sum of s^j over the sizes, for j in 0..=deg (entry 0 is left at zero)
fn power_sums(sizes: List<Z>, deg: Int) -> List<Z>:
    let p = prod_linear(sizes).rev.to_list().resized(deg + 1, Z(0))
    // -x p'(x) / p(x)
    let r = convolve([-Z(i) * p[i] for i in 0..=deg], inv(p, deg + 1))
    r[..deg + 1].to_list()

// res[i] = [x^(hi - lo)] of c * prod(1 - u x for u in lo..=i); `c` has hi - lo + 1 terms
fn descend(seg: List<List<Z>>, node: Int, lo: Int, hi: Int, c: List<Z>, mut res: List<Z>):
    if hi - lo == 1:
        res[lo] = c[1] - Z(lo) * c[0]
        return
    let mid = (lo + hi) / 2
    descend(seg, 2 * node, lo, mid, c[..mid - lo + 1], mut res)
    let full = convolve(c, seg[2 * node])
    descend(seg, 2 * node + 1, mid, hi, full[mid - lo..=hi - lo], mut res)

// Walk the suffix tree in LCP order, recording (top_size, bottom_size) per edge.
fn suffix_tree_edges(s: Str) -> (below: List<Z>, above: List<Z>):
    let n = s.len
    let sa = suffix_array(s)
    let lcp = lcp_array(s, sa)
    var below = List<Z>()
    var above = List<Z>()
    var open: List<OpenNode> = []
    for i in 0..n:
        let next = if i + 1 < n: lcp[i] else: 0
        var (depth, size) = (n - sa[i], 0)
        while let top = open.last and top.depth > next:
            if depth > top.depth:
                below.push(Z(size))
                size += depth - top.depth
                above.push(Z(size))
            let prev = open.pop()!
            depth = prev.depth
            size += prev.size
        if depth > next:
            below.push(Z(size))
            size += depth - next
            above.push(Z(size))
        if i + 1 < n:
            if let top = open.last and top.depth == next:
                open[open.len - 1] = OpenNode(top.depth, top.size + size)
            else:
                open.push(OpenNode(next, size))
    (below, above)

@!multitest
let n: 1..=2e5 = read()
let s: Str = read()
let (below, above) = suffix_tree_edges(s)

let top = power_sums(above, n)
let bottom = power_sums(below, n)
let sums = [top[j] - bottom[j] for j in 0..=n]
var eval_tree = List.filled(4 * n, List<Z>())
product(mut eval_tree, 1, 0, n, [Z(u) for u in 0..n], n + 1)
var ans = List.filled(n, Z(0))
descend(eval_tree, 1, 0, n, sums, mut ans)
print([(ans[i] * comb.inv_fact(i + 1)).v for i in 0..n])
