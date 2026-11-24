// Source: DMPG '16 S5 (Pizza Bag), https://dmoj.ca/problem/dmpg16s5
// limits: 1400 ms, 256 MB
// Problem: n slices around a circular pizza (n <= 1e5) with tastes in [-1e8, 1e8], at
//   least one positive. Print the largest total of at most k consecutive slices (k <= n).
// Solution: unroll the circle twice; a run ending at slice p is the prefix sum after p
//   minus the smallest prefix sum at the k cuts before it: a sliding-window minimum, kept
//   in a deque of (cut, prefix) with prefix sums rising towards the back. O(n).
let n: 1..=1e5 = read()
let k: 1..=n = read()
let taste: [0..n]-1e9..=1e9 = read()

var window = Deque<(cut: Int, pre: Int)>()  // prefix sums rising towards the back
var pre = 0
var best = 0
for cut in 0..2 * n:
    while let (_, last) = window.last and last >= pre: window.pop()
    window.push((cut, pre))
    if window.first!.cut < cut + 1 - k: window.pop_front()
    pre += taste[cut % n]
    best max= pre - window.first!.pre
print(best)
