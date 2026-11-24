// Source: UCRPC F21 E (Feeding Friendsy), https://dmoj.ca/problem/ucrpc21e
// limits: 2500 ms, 256 MB
// Problem: a game of T time units; the upper and the lower mouth each open on n (resp. m)
//   disjoint sorted intervals [a, b], each worth 1 or (rainbow) 3 points per goal. Each time
//   unit you throw at most one ball into one open mouth. Print the most points possible.
// Solution: each time unit independently takes the better of the two mouths, so the answer
//   is the sum over time of max(upper rate, lower rate). One event per interval endpoint
//   (+worth at a, -worth at b + 1), sorted; the sweep adds gap * max rate. O((n + m) log).
// Bounds: len: 1..=1e9; n, m: 0..=1e5
// original: corpus/submissions/3944571.cpp
let (len, n, m): (1..=1e9, 0..=1e5, 0..=1e5) = read()
domain Mouth = [2]
type Window = (a: 1..=len, b: 1..=len, rainbow: Bool)
let windows: [Mouth]List<Window> = [List(read<[n]Window>()), List(read<[m]Window>())]

let worth = (w: Window) => if w.rainbow: 3 else: 1
var changes = List<(at: Int, mouth: Mouth, d: Int)>()
for mouth in Mouth:
    for w in windows[mouth]:
        changes.push((at: w.a, mouth:, d: worth(w)))
        changes.push((at: w.b + 1, mouth:, d: -worth(w)))
changes.sort()
var rate: [Mouth]Int = 0
var (score, last) = (0, 1)
for c in changes:
    score += (c.at - last) * rate.max()!
    rate[c.mouth] += c.d
    last = c.at
print(score)
