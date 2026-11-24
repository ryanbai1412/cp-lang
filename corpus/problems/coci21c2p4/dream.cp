// COCI 2021/22 C2 P4 Magneti: count placements of n magnets in l slots with no attraction.
// Insert magnets by increasing radius: ways[j, k] counts j ordered tight blocks using k cells.
// A magnet of radius r opens a block (j + 1 places, 1 cell), joins a block's end (2j ways,
// r cells) or glues two neighbours (j - 1 ways, 2r - 1 cells); the l - k spare cells of the
// final block spread over n + 1 gaps.
type Z = Mod<1_000_000_007>
let (n, l): (Int, Int) = read()
domain Magnet = 0..n
let radius: [Magnet]Int = read()
domain Blocks = 0..=n + 1
domain Cells = 0..=l

var ways: [Blocks, Cells]Z = Z(0)
ways[Blocks(0), Cells(0)] = Z(1)
for r in radius.sorted():
    var next: [Blocks, Cells]Z = Z(0)
    for j in Blocks:
        for k in Cells:
            let w = ways[j, k]
            if w == 0: continue
            if let a = Blocks.at(j.ord + 1), let b = Cells.at(k.ord + 1): next[a, b] += w * Z(j.ord + 1)
            if let b = Cells.at(k.ord + r): next[j, b] += w * Z(2 * j.ord)
            if j.ord >= 2:
                if let a = Blocks.at(j.ord - 1), let b = Cells.at(k.ord + 2 * r - 1): next[a, b] += w * Z(j.ord - 1)
    ways = next

let comb = Comb<Z>(l + n)
print(Cells.map(k => ways[Blocks(1), k] * comb.choose(l - k.ord + n, n)).sum())
