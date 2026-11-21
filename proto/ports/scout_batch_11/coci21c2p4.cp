// COCI 2021/2022 Contest 2 P4, Magneti: n magnets go into l cells in a
// row; two magnets attract when their distance is below either radius.
// The number of placements with no attraction, modulo 1e9+7.
// Insert the magnets by increasing radius, so the one being placed decides
// its distance to both neighbours. Blocks of magnets packed as tightly as
// allowed grow and merge; the cells left over are shared out at the end.
// original: corpus/submissions/4226760.cpp

const P = 1000000007
type Z = Mod<P>

let (n, l): (Int, Int) = read()  // n <= 50, l <= 1e4
domain Magnet = 0..n
let radius: [Magnet]Int = read()

// ways[j][k]: the magnets so far form j blocks, in a fixed order left to right, that take k cells
var ways = [[Z.zero for _ in 0..=l] for _ in 0..=n + 1]
ways[0][0] = 1
for r in sorted(radius):
    var next = [[Z.zero for _ in 0..=l] for _ in 0..=n + 1]
    for j in 0..=n:
        for k in 0..=l:
            let w = ways[j][k]
            if w == 0: continue
            if k + 1 <= l: next[j + 1][k + 1] += w * Z(j + 1)                             // a block of its own, in any of j + 1 places
            if k + r <= l: next[j][k + r] += w * Z(2 * j)                                 // at either end of a block
            if j >= 2 and k + 2 * r - 1 <= l: next[j - 1][k + 2 * r - 1] += w * Z(j - 1)  // joining two neighbouring blocks
    ways = next

// One block of k cells: the l - k free cells go into the n + 1 gaps around the magnets.
let comb = Comb<P>.new(l + n)
var total: Z = 0
for k in 1..=l: total += ways[1][k] * comb.choose(l - k + n, n)
print(total)
