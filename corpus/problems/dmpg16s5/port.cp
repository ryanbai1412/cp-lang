// DMPG 2016 S5 Pizza Bag: the tastiest run of at most k consecutive slices
// of a circular pizza. On the pizza unrolled twice, a run ending at slice p
// is a prefix sum minus the smallest prefix sum among the k cuts before it:
// a sliding-window minimum.
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
