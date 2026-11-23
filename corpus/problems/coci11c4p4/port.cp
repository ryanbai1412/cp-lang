// COCI 2011 Ograda: a fence of n boards is to be rebuilt from n new boards.
// The new fence must rise wherever the old one rises and fall wherever it
// falls, and the sum of the height differences of neighbours must be as large
// as possible. Print that sum and one such fence.
// original: corpus/submissions/5325112.cpp
let n: 2..=3e5 = read()
domain I = 1..=n
let old: [I]1..=1e9 = read()
assert all(old[i] != old[j] for i in I if let j = I.at(i + 1))
let boards: [I]1..=1e9 = read()

// A board counts once in the sum for each lower neighbour and once against it
// for each higher one, so the tall boards go where that pull is largest. The
// boards on a slope pull nothing; they only have to keep the slope, so rising
// slopes are filled left to right before falling ones right to left.
type Rank = (pull: -2..=2, falling: Bool, turn: Int)
fn rank(i: I) -> Rank:
    var pull = 0
    for side in [i.prev, i.next]:
        if let j = side: pull += if old[i] > old[j]: 1 else: -1
    let falling = pull > 0 or (pull == 0 and old[i.prev!] > old[i])
    (pull, falling, if falling: -i.idx else: i.idx)

var pile = sorted([b for b in boards], by: b => -b)  // the shortest board is on top
var fence: [I]Int = 0
for i in sorted([i for i in I], by: i => rank(i)): fence[i] = pile.pop()!
print(sum(abs(fence[i] - fence[j]) for i in I if let j = I.at(i + 1)))
print(fence)
