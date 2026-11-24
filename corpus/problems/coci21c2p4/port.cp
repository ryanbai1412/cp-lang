// Source: COCI 2021/2022 Contest 2 (Magneti), https://dmoj.ca/problem/coci21c2p4
// limits: 1000 ms, 512 MB
// Problem: n <= 50 distinct magnets with radii r_i go into a row of l <= 1e4 slots (n <= l), at
//   most one per slot; a magnet attracts any other strictly closer than its own radius. Count the
//   placements with no attraction, modulo 1e9+7.
// Solution: insert magnets by increasing radius, so the newcomer's radius sets its gap to each
//   neighbour. ways[j][k]: j ordered blocks packed as tightly as allowed, using k cells. A magnet
//   of radius r opens a block (j + 1 places, 1 cell), joins either end of a block (2j ways, r
//   cells) or joins two neighbouring blocks (j - 1 ways, 2r - 1 cells). With one block of k
//   cells, the l - k free cells spread over n + 1 gaps: C(l - k + n, n). O(n^2 l).
// original: corpus/submissions/4226760.cpp

const P = 1000000007
type Z = Mod<P>

let (n, l): (Int, Int) = read()  // n <= 50, l <= 1e4
domain Magnet = 0..n
let radius: [Magnet]Int = read()

// ways[j][k]: the magnets so far form j blocks, in a fixed order left to right, that take k cells
var ways = [[Z.zero for _ in 0..=l] for _ in 0..=n + 1]
ways[0][0] = 1
for r in radius.sorted():
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
let comb = Comb<Mod<P>>(l + n)
var total: Z = 0
for k in 1..=l: total += ways[1][k] * comb.choose(l - k + n, n)
print(total)
