// Source: IOI 2019 Day 1 P1 (Arranging Shoes), https://dmoj.ca/problem/ioi19p1
// Problem: 2n shoes in a row (n <= 1e5), -s a left shoe of size s and +s a right one. With
//   swaps of adjacent shoes, make every pair of positions 2i, 2i+1 a left then a right shoe of
//   one size. Print the fewest swaps.
// Solution: fix the leftmost shoe: its partner should be the nearest unused shoe of the same
//   size and other foot (per size, a queue of shoes still waiting). Pull the partner next to
//   it, one swap per shoe still standing between them, plus one swap if the pair is the wrong
//   way round, then forget both. A SegTree of standing shoes counts them: O(n log n).
// original: corpus/submissions/5661847.cpp
let n = read<1..=100_000>()
domain P = [2 * n]
domain Shoe = -n..=n
let s = read<[P]Shoe>()
var next_of: [Shoe]Deque<P> = Deque()
for (i, x) in s.items:
    next_of[x].push(i)
var standing = Fenwick([P] i => 1)
var paired: [P]Bool = false
var swaps = 0
for (i, x) in s.items if not paired[i]:
    next_of[x].pop_front()
    let j = next_of[Shoe(-Int(x))].pop_front()!
    paired[j] = true
    standing.add(j, -1)
    swaps += standing.fold(i.r..j.l) + Int(x > 0)
print(swaps)
