// NOI 2006 "Millennium Bug": add the fewest cells so that both side edges of
// the shape become combs, alternating protruding and recessed runs.
enum Side = Low | High

fn comb(depth: List<Int>) -> Int:
    domain I = [depth.len]
    domain H = 0..1005

    @memo(array)
    fn best(i: I, h: H, prev: Side, cur: Side) -> Int:
        if depth[i] > h:
            return inf
        let j = i.next ?? return if cur == Low: 0 else: inf
        let keep = h - depth[j] + best(j, h, cur, cur)
        let nudge = if prev == cur: inf else:
            let step = if cur == Low: -1 else: 1
            H.at(h + step).map(g => step + best(i, g, prev, cur)) ?? inf
        let turn_to = if cur == Low: max(h + 1, depth[j]) else: min(h - 1, depth[j])
        let turn = H.at(turn_to).map(g => g - depth[j] + best(j, g, cur, cur.flip())) ?? inf
        min(keep, nudge, turn)

    best(I.first!, depth[I.first!], High, Low)

let n = read<1..=1000>()
let rows = read<[n](l: 0..=1000, r: 0..=1000)>()
let reach = rows.max(of: .l)!
print(comb(rows.map(x => reach - x.l)) + comb(rows.map(.r)))
