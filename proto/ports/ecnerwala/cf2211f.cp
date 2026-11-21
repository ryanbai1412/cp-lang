// Codeforces 2211 F.
// Repeatedly halving a segment of length L into floor(L/2) and floor((L-1)/2)
// produces at most two consecutive segment lengths at every depth `cost`:
// `big` and `small = big - 1`, tracked in `Layer`.
// `ways(a) = C(n - a + m, m - 1)` counts non-decreasing length-m arrays in
// [1..=n] whose span covers at least `a` elements. At depth `cost >= 2`, each
// segment of length `len` contributes from its inner/outer half-cuts:
//   cnt * (ways(1) - ways(inner_cut + 2)) + (cnt - 1) * (ways(len + 2) - ways(outer_cut + 2)),
// where `big` uses `(inner, outer) = ((len - 1) / 2, len / 2)` and `small` swaps them.
type Z = Mod<676_767_677>

struct Layer:
    big_len: Int
    small_len: Int
    big_cnt: Int
    small_cnt: Int

    fn halve(self) -> Layer:
        let next_big = big_len / 2
        var (nb, ns) = (0, 0)
        for (len, cnt) in [(big_len, big_cnt), (small_len, small_cnt)]:
            for d in 0..=1:
                if (len - d) / 2 == next_big: nb += cnt else: ns += cnt
        Layer(next_big, next_big - 1, nb, ns)

@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
let comb = Comb<Z>(n + m + 1)
fn ways(a: Int) -> Z = comb.choose(n - a + m, m - 1)

fn contrib(len: Int, cnt: Int, inner_cut: Int, outer_cut: Int) -> Z =
    Z(cnt) * (ways(1) - ways(inner_cut + 2)) + Z(cnt - 1) * (ways(len + 2) - ways(outer_cut + 2))

var ans: Z = 0
var layer = Layer(n, n - 1, 1, 0)
var cost = 1

while layer.big_len > 0:
    if cost == 1:
        ans += Z(layer.big_cnt + layer.small_cnt) * ways(1)
    else:
        if layer.small_cnt == 0:
            layer = Layer(layer.big_len, layer.big_len, layer.big_cnt - 1, 1)
        let c = Z(cost)
        ans += c * contrib(layer.big_len, layer.big_cnt, (layer.big_len - 1) / 2, layer.big_len / 2)
        if layer.small_len > 0:
            ans += c * contrib(
                layer.small_len,
                layer.small_cnt,
                layer.small_len / 2,
                (layer.small_len - 1) / 2,
            )
    if layer.small_len == 0: break
    layer = layer.halve()
    cost += 1

print(ans)
