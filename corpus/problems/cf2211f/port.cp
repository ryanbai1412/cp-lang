// Source: Codeforces 2211F (Learning Binary Search), https://codeforces.com/contest/2211/problem/F
// Problem: f(a, k) counts the calls of the given recursive binary search for k on a sorted
//   a[1..=n] (0 if k is absent). Over all nondecreasing a of length n with values in 1..=m, print
//   the sum of f(a, k) for k in 1..=m, mod 676767677. n, m <= 1e6; multitest, sums <= 1e6.
// Solution: k's copies form a block and the search stops at the first mid inside it, so f counts
//   the depths whose segment still holds the whole block. Sum, over depths and segments, the
//   pairs (a, k) whose block lies in the segment, by inclusion-exclusion over windows with
//   ways(w) = C(n - w + m, m - 1), the arrays constant on a fixed window of w cells. Halving
//   leaves at most two segment lengths per depth (`Layer`), so each of the O(log n) depths costs
//   O(1) with factorials: O(n + m) per test.

type Z = Mod<676_767_677>

struct Part:
    len: Int
    cnt: Int
    fn upper_half(self) -> Int = len / 2
    fn lower_half(self) -> Int = (len - 1) / 2

struct Layer:
    big: Part
    small: Part

    fn normalized(self) -> Layer =
        if small.cnt == 0: Layer(Part(big.len, big.cnt - 1), Part(big.len, 1))
        else: self

    fn halve(self) -> Layer:
        let next_big = big.upper_half()
        var (nb, ns) = (0, 0)
        for p in [big, small]:
            for h in [p.upper_half(), p.lower_half()]:
                if h == next_big: nb += p.cnt else: ns += p.cnt
        Layer(Part(next_big, nb), Part(next_big - 1, ns))

fn deeper_layers(n: Int) -> List<Layer>:
    var out: List<Layer> = []
    var cur = Layer(Part(n, 1), Part(n - 1, 0))
    while cur.small.len > 0:
        cur = cur.halve()
        if cur.big.len == 0: break
        out.push(cur.normalized())
    out

@!multitest
let (n, m): (3..=1_000_000, 3..=1_000_000) = read()
let comb = Comb<Z>(n + m + 1)
fn ways(a: Int) -> Z = comb.choose(n - a + m, m - 1)

fn contrib(p: Part, inner: Int, outer: Int) -> Z =
    Z(p.cnt) * (ways(1) - ways(inner + 2)) + Z(p.cnt - 1) * (ways(p.len + 2) - ways(outer + 2))

var ans = ways(1)
for (idx, layer) in deeper_layers(n).items:
    let cost = Z(idx + 2)
    ans += cost * contrib(layer.big, layer.big.lower_half(), layer.big.upper_half())
    if layer.small.len > 0:
        ans += cost * contrib(layer.small, layer.small.upper_half(), layer.small.lower_half())

print(ans)
