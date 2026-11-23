// coci21c2p4 Magneti: place n magnets on l slots so no magnet attracts another (the gap to a
// neighbour is at least the larger radius). Insert magnets smallest radius first, so each newcomer
// sets both its gaps: ways[j, k] counts j ordered blocks packed tight into k cells. A magnet opens
// a block (j + 1 places), joins either end of one (2j, r cells) or two neighbours (j - 1, 2r - 1).
// One block of k cells leaves l - k free cells for the n + 1 gaps: C(l - k + n, n). O(n^2 l).
type Z = Mod<1e9 + 7>
let (n, l) = read<(1..=50, 1..=1e4)>()
let radii = read<[n]1..=1e4>().sorted()
domain Blocks = 0..=n
domain Cells = 0..=l
domain State = (Blocks, Cells)

var ways: [State]Z = 0
ways[0, 0] = 1
for r in radii:
    var next: [State]Z = 0
    for (j, k) in State:
        let w = ways[j, k]
        if w == 0: continue
        fn add(dj: Int, dk: Int, times: Int):
            if let t = State.at((Int(j) + dj, Int(k) + dk)): next[t] += w * times
        add(1, 1, Int(j) + 1)
        add(0, r, 2 * Int(j))
        add(-1, 2 * r - 1, Int(j) - 1)
    ways = next

let comb = Comb<Z>(l + n)
print(Cells.sum(of: k => ways[1, k] * comb.choose(l - Int(k) + n, n)))
