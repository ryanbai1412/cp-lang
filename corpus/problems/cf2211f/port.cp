// Codeforces 2211 F.
// Repeatedly halving a segment of length L splits it into `upper_half = L / 2`
// and `lower_half = (L - 1) / 2`. At every depth `cost`, all segments have at
// most two consecutive lengths `big` and `small = big - 1`, stored as
// `Layer(big: Part, small: Part)`.
// `ways(a) = C(n - a + m, m - 1)` counts non-decreasing length-m arrays in
// `1..=n` whose span covers at least `a` elements. Depth 1 contributes
// `ways(1)` for the whole array. At each deeper depth `cost >= 2`:
//   - When `small.cnt == 0`, all segments have equal length `big.len`; moving
//     one segment into `small` at length `big.len` preserves the multiset while
//     splitting the `cnt - 1` gap terms into the same two-part formula.
//   - Each part contributes `cnt * (ways(1) - ways(inner + 2)) +
//     (cnt - 1) * (ways(len + 2) - ways(outer + 2))`, where `big` uses
//     `(inner, outer) = (lower_half, upper_half)` and `small` swaps them.

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
let (n, m): (1..=200_000, 1..=200_000) = read()
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
