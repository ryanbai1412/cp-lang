// COCI 2011 Ograda: a fence of n boards is to be rebuilt from n new boards.
// The new fence must rise wherever the old one rises and fall wherever it
// falls, and the sum of the height differences of neighbours must be as large
// as possible. Print that sum and one such fence.
// original: corpus/submissions/5325112.cpp
let n: 2..=3e5 = read()
domain I = 1..=n
let old: [I]1..=1e9 = read() where all(x != y for (x, y) in old.adjacent)           // WISH S17
let boards: [I]1..=1e9 = read()

// A board counts once in the sum for each lower neighbour and once against it
// for each higher one, so the tall boards go where that pull is largest. The
// boards on a slope pull nothing; they only have to keep the slope, so rising
// slopes are filled left to right before falling ones right to left.
fn rank(i: I) -> (-2..=2, Bool, Int):
    let pull = sum(sign(old[i] - old[j]) for j in i.neighbours)                     // WISH S17, G20
    let falling = pull > 0 or (pull == 0 and old[i.prev!] > old[i])
    (pull, falling, if falling: -i.idx else: i.idx)

var fence: [I]Int                       // every cell is written below                // WISH S10
for (i, board) in zip(sorted(I, by: rank), boards.sorted()): fence[i] = board        // WISH S16, G21
print(sum(abs(x - y) for (x, y) in fence.adjacent))                                 // WISH S17
print(fence)
