// CF 2037F Ardent Flames: binary search the number of attacks q. With q attacks enemy i
// dies iff |p - x_i| <= m - ceil(h_i / q); some p must lie in at least k such intervals.
@!multitest
let (n, m, k): (1..=1e5, 1..=1e9, 1..=n) = read()
domain E = [n]
let h = read<[E]1..=1e9>()
let x = read<[E]1..=1e9>()

fn enough(q: Int) -> Bool:
    var ivs: List<(l: Int, r: Int)> = []
    for i in E:
        let reach = m - h[i].ceil_div(q)
        if reach >= 0:
            ivs.push((l: x[i] - reach, r: x[i] + reach))
    if ivs.len < k: return false
    let (domain P, pts) = compress([iv.l for iv in ivs])
    var cover = SegTree<P, Int>(max, fill: 0, act: add)
    for iv in ivs:
        // every compressed left end inside [l, r] is a point covered by this interval
        let lo = pts.at(iv.l)
        let hi = pts.last_le(iv.r)!
        cover.apply(lo..=hi, 1)
    cover.all >= k

print((1..=1e9).first_true(enough) ?? -1)
