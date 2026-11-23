// NOI 2006 "Millennium Bug": add the fewest cells so that both side edges of
// the shape become combs, alternating protruding and recessed runs. Each side
// is a DP over (row, edge height, previous run kind, current run kind).
enum Side = Low | High

fn comb(depth: List<Int>) -> Int:
    domain I = [depth.len]
    domain H = 0..1005

    @memo(array)
    fn best(i: I, h: H, prev: Side, cur: Side) -> Int:
        if depth[Int(i)] > Int(h):
            return inf
        let j = i.next ?? return if cur == Low: 0 else: inf
        let keep = Int(h) - depth[Int(j)] + best(j, h, cur, cur)
        let step = if cur == Low: -1 else: 1
        let nudge = if prev == cur: inf else: (h + step).map(g => step + best(i, g, prev, cur)) ?? inf
        let turn_to = if cur == Low: max(Int(h) + 1, depth[Int(j)]) else: min(Int(h) - 1, depth[Int(j)])
        let flip = if cur == Low: High else: Low
        let turn = H.at(turn_to).map(g => Int(g) - depth[Int(j)] + best(j, g, cur, flip)) ?? inf
        min(keep, nudge, turn)

    best(I.first!, H(depth[Int(I.first!)]), High, Low)

let n = read<1..=1000>()
let rows = read<[n](l: 0..=1000, r: 0..=1000)>()
let reach = rows.max(by: x => x.l)!.l
print(comb(rows.map(x => reach - x.l).collect()) + comb(rows.map(x => x.r).collect()))
