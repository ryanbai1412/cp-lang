// IOI 2019 Shoes: adjacent swaps to arrange shoes as (left, right) pairs of equal size.
// Greedy from the left: the first unmatched shoe pairs with the nearest unmatched shoe of the
// opposite foot; moving it next to it costs the unmatched shoes in between, plus one if the
// first shoe is a right shoe.
let n = read<1..=100_000>()
domain P = [2 * n]
domain Shoe = -n..=n
let s = read<[P]Shoe>()
var waiting: [Shoe]Deque<P> = Deque()
for (i, x) in s.items: waiting[x].push(i)
var matched = Fenwick<P, Int>()
var swaps = 0
for (i, x) in s.items if matched.get(i) == 0:
    waiting[x].pop_front()
    let j = waiting[Shoe(-Int(x))].pop_front()!
    swaps += (i.r..j.l).len - matched.fold(i.r..j.l) + Int(x > 0)
    matched.add(j, 1)
print(swaps)
