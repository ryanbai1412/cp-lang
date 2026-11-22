// ioi19p1 Arranging Shoes: fewest adjacent swaps to pair every left shoe with a right shoe of the
// same size just after it. Greedily pair the leftmost unpaired shoe with the nearest matching
// one; the swaps are the shoes still standing between them, plus one if the first is a right.
let n = read<1..=100_000>()
let s = read<[2 * n](-n..=n)>()
var next_of: [-n..=n]Deque<Int> = [-n..=n] v => Deque()
for (i, x) in s.items:
    next_of[x].push(i)
var standing = Fenwick([2 * n] i => 1)
var paired: [2 * n]Bool = false
var swaps = 0
for (i, x) in s.items:
    if paired[i]:
        continue
    next_of[x].pop_front()
    let j = next_of[-x].pop_front()!
    paired[j] = true
    standing.add(j, -1)
    swaps += standing.sum(i + 1..j) + Int(x > 0)
print(swaps)
