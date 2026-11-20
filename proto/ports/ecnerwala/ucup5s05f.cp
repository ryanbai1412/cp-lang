// Universal Cup 5 Stage 5 (Chengdu) F: answer[k] = sum of choose(size, k) over
// suffix-tree edges, bottom end minus top end. The sizes reach n^2, so take their
// power sums (a logarithmic derivative) and turn those into binomial sums with
// a transposed multipoint evaluation:
//   answer[k] = [x^k] (power_sums(x) * prod(1 - u x for u in 0..k)) / k!
use lib.math.poly.{Z, convolve, inv_trunc}
use std.strings.{suffix_array, lcp_array}

const MOD = 998244353
let comb = Comb<MOD>()

// tree[node] = prod(1 - roots[i] x for i in lo..hi), cut to `keep` terms
fn product(mut tree: List<List<Z>>, node: Int, lo: Int, hi: Int, roots: List<Z>, keep: Int):
    if hi - lo == 1:
        tree[node] = [Z(1), -roots[lo]]
        return
    let mid = (lo + hi) / 2
    product(mut tree, 2 * node, lo, mid, roots, keep)
    product(mut tree, 2 * node + 1, mid, hi, roots, keep)
    let p = convolve(tree[2 * node], tree[2 * node + 1])
    tree[node] = if p.len > keep: p[..keep] else: p

// sum of s^j over the sizes, for j in 0..=n (entry 0 is left at zero)
fn power_sums(sizes: List<Z>, n: Int) -> List<Z>:
    var tree = List.filled(4 * sizes.len, List<Z>())
    product(mut tree, 1, 0, sizes.len, sizes, n + 1)
    let p = tree[1].resized(n + 1, 0)
    // -x p'(x) / p(x)
    let r = convolve([-Z(i) * p[i] for i in 0..=n], inv_trunc(p, n + 1))
    r[..n + 1]

// out[i] = [x^(hi - lo)] of c * prod(1 - u x for u in lo..=i); `c` has hi - lo + 1 terms
fn descend(tree: List<List<Z>>, node: Int, lo: Int, hi: Int, c: List<Z>, mut out: List<Z>):
    if hi - lo == 1:
        out[lo] = c[1] - Z(lo) * c[0]
        return
    let mid = (lo + hi) / 2
    descend(tree, 2 * node, lo, mid, c[..mid - lo + 1], mut out)
    let full = convolve(c, tree[2 * node])
    descend(tree, 2 * node + 1, mid, hi, full[mid - lo..=hi - lo], mut out)

@!multitest
let n: Int = read()
let s: Str = read()
let sa = suffix_array(s)
let lcp = lcp_array(s, sa)

// Walk the suffix tree with a stack of open internal nodes (depth, sizes below).
var below = List<Z>()
var above = List<Z>()
var open_depth = List<Int>()
var open_size = List<Int>()
for i in 0..n:
    let next = if i + 1 < n: lcp[i] else: -1
    var (depth, size) = (n - sa[i], 0)
    while true:
        let top = open_depth.last ?? -1
        let parent = if top > next: top else: max(next, 0)
        if depth > parent:
            below.push(Z(size))
            size += depth - parent
            above.push(Z(size))
        if top > next:
            // the open node ends here: it becomes the subtree being closed
            depth = open_depth.pop()!
            size += open_size.pop()!
        elif top == next and next >= 0:
            open_size.last! += size
            break
        else:
            if next >= 0:
                open_depth.push(next)
                open_size.push(size)
            break

let top = power_sums(above, n)
let bottom = power_sums(below, n)
let sums = [top[j] - bottom[j] for j in 0..=n]
var tree = List.filled(4 * n, List<Z>())
product(mut tree, 1, 0, n, [Z(u) for u in 0..n], n + 1)
var out = List.filled(n, Z(0))
descend(tree, 1, 0, n, sums, mut out)
// a `List<Z>` prints one value per line; the answers go on one line
print([(out[i] * comb.inv_fact(i + 1)).v for i in 0..n])
