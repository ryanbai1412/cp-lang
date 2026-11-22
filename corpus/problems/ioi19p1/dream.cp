// ioi19p1 Arranging Shoes: fewest adjacent swaps to pair every left shoe with a right shoe of the
// same size just after it. Greedily pair the leftmost unpaired shoe with the nearest matching
// one; the swaps are the shoes still standing between them, plus one if the first is a right.
let n = read<1..=100_000>()
domain P = [2 * n]
let s = read<[P](-n..=n)>()
var next_of: [-n..=n]Deque<P> = [-n..=n] v => Deque()
for (i, x) in s.items:
    next_of[x].push(i)
var standing = Fenwick([P] i => 1)
var paired: [P]Bool = false
var swaps = 0
for (i, x) in s.items if not paired[i]:
    next_of[x].pop_front()
    let j = next_of[-x].pop_front()!
    paired[j] = true
    standing.add(j, -1)
    swaps += standing.sum(i.r..j.l) + (x > 0).int
print(swaps)
