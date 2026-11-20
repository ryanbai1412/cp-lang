// original: 3997206 (cmt21r1xym) -- first draft, written from docs/one-page-cheat-sheet.md only; does not build on proto/cpc
const MOD = 998244353
type Z = Mod<MOD>
let n: Int = read()
domain I = 0..n
let g: [I, I]Char = read()
let comb = Comb<MOD>(n)
fn solve(cnt: [I]Int) -> List<(Int, Z, Int)>:
    let a = sorted(cnt, by: x => -x)
    let pre = prefix(a)
    let total = sum(a)
    var out: List<(Int, Z, Int)> = []
    for i in 0..=n:
        var top = 0
        if i > 0: top = pre.sum(I.first..=I.from_idx(i - 1)!)
        let mx = top + n * (n - i) - (total - top)
        var ways: Z = 1
        if i > 0 and i < n:
            let c = a.raw[i - 1]
            let lo = sum(1 for x in a if x > c)
            let hi = sum(1 for x in a if x >= c)
            ways = comb.ncr(hi - lo, i - lo)
        out.push((mx, ways, i))
    return out
let rows: [I]Int = [I] i => sum(1 for j in I if g[i, j] == '1')
let cols: [I]Int = [I] j => sum(1 for i in I if g[i, j] == '1')
var ans: Z = 0
for (ma, wa, xa) in solve(rows):
    for (mb, wb, xb) in solve(cols):
        if ma + mb - xa * xb - (n - xa) * (n - xb) == n * n: ans += wa * wb
print(ans)
