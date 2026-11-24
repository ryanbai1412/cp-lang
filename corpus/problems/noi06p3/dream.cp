// NOI 2006 "Millennium Bug": add the fewest cells so that both side edges of the shape become
// combs: runs of equal depth, alternately recessed (Low) and protruding (High), starting and
// ending recessed, each protrusion deeper than the recesses beside it. Cells are only added, so a
// row's depth only grows; a DP over (row, depth, side).
enum Side = Low | High
let never = 1 << 60  // an impossible comb; sums of a few stay in range

let n = read<1..=1_000_000>()
domain Col = 0..n
let rows = read<[Col](l: 0..=1_000_000, r: 0..=1_000_000)>()

fn comb(depth: [Col]Int) -> Int:
    domain H = 0..=depth.max()! + 1
    let other = (s: Side) => if s == Low: High else: Low

    // the cheapest comb of rows i.. with row i on side s at depth h, or, when `past`, at a depth
    // strictly deeper (High) or shallower (Low) than h
    @memo(array)
    fn best(i: Col, h: H, s: Side, past: Bool) -> Int:
        if past:
            let g = (if s == High: h.next else: h.prev) ?? return never
            return min(best(i, g, s, false), best(i, g, s, true))
        if Int(h) < depth[i]: return never
        let here = Int(h) - depth[i]
        let j = i.next ?? return if s == Low: here else: never
        here + min(best(j, h, s, false), best(j, h, other(s), true))

    H.all.map(h => best(Col.first!, h, Low, false)).min()!

let reach = rows.map(.l).max()!
print(comb(rows.map(x => reach - x.l)) + comb(rows.map(.r)))
