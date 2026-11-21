// Universal Cup 5 Stage 5 (Chengdu) F: answer[k] = sum of choose(size, k) over
// suffix-tree edges, bottom end minus top end. The sizes reach n^2, so take their
// power sums (a logarithmic derivative) and turn those into binomial sums with
// a transposed multipoint evaluation:
//   answer[k] = [x^k] (power_sums(x) * prod(1 - u x for u in 0..k)) / k!
use lib.math.poly.{Z, convolve, inv_trunc}
use std.strings.{suffix_array, lcp_array}

let comb = Comb<Z>()

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
    var seg = List.filled(4 * sizes.len, List<Z>())
    product(mut seg, 1, 0, sizes.len, sizes, deg + 1)
    let p = seg[1].resized(deg + 1, 0)
    // -x p'(x) / p(x)
    let r = convolve([-Z(i) * p[i] for i in 0..=deg], inv_trunc(p, deg + 1))
    r[..deg + 1]

// res[i] = [x^(hi - lo)] of c * prod(1 - u x for u in lo..=i); `c` has hi - lo + 1 terms
fn descend(seg: List<List<Z>>, node: Int, lo: Int, hi: Int, c: List<Z>, mut res: List<Z>):
    if hi - lo == 1:
        res[lo] = c[1] - Z(lo) * c[0]
        return
    let mid = (lo + hi) / 2
    descend(seg, 2 * node, lo, mid, c[..mid - lo + 1], mut res)
    let full = convolve(c, seg[2 * node])
    descend(seg, 2 * node + 1, mid, hi, full[mid - lo..=hi - lo], mut res)

@!multitest
let n: Int = read()
let s: Str = read()
let sa = suffix_array(s)
let lcp = lcp_array(s, sa)

// Walk the suffix tree with a stack of open internal nodes (depth, sizes below).
var below = List<Z>()
var above = List<Z>()
var open: [0..n](depth: Int, size: Int) = (0, 0)
var open_len = 0

for i in 0..n:
    let next = if i + 1 < n: lcp[i] else: -1
    var (depth, size) = (n - sa[i], 0)
    while true:
        let top = if open_len > 0: open[open_len - 1].depth else: -1
        let parent = if top > next: top else: max(next, 0)
        if depth > parent:
            below.push(Z(size))
            size += depth - parent
            above.push(Z(size))
        if top > next:
            open_len -= 1
            depth = open[open_len].depth
            size += open[open_len].size
        elif top == next and next >= 0:
            open[open_len - 1].size += size
            break
        else:
            if next >= 0:
                open[open_len] = (next, size)
                open_len += 1
            break

let top = power_sums(above, n)
let bottom = power_sums(below, n)
let sums = [top[j] - bottom[j] for j in 0..=n]
var eval_tree = List.filled(4 * n, List<Z>())
product(mut eval_tree, 1, 0, n, [Z(u) for u in 0..n], n + 1)
var ans = List.filled(n, Z(0))
descend(eval_tree, 1, 0, n, sums, mut ans)
print([(ans[i] * comb.inv_fact(i + 1)).v for i in 0..n])
