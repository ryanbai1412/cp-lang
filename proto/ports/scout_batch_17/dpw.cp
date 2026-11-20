// Educational DP W, Intervals: choose a 0/1 string of length n. Interval i
// pays a (maybe negative) score when the string has a 1 somewhere in l..=r.
// Maximise the total.
// original: corpus/submissions/4133460.cpp
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain Pos = 1..=n
domain I = 1..=m
type Interval = (l: Pos, r: Pos, score: -1e9..=1e9)

fn interval() -> Interval:
    let (a, b) = (rand(1, n), rand(1, n))
    (Pos(min(a, b)), Pos(max(a, b)), rand(-1e9, 1e9))

fn ordered() -> [I]Interval:
    [I] _ => interval()

let intervals: [I]Interval = read(gen: ordered)

var ending: [Pos]List<(Pos, Int)> = []
for (l, r, score) in intervals: ending[r].push((l, score))

// best.max(p, p): the most a string can earn whose last 1 is at p, counting
// the intervals that end at or before the sweep line. An interval ending
// later that holds p also holds the sweep line, so it is paid when it ends.
var best = AddMax<Pos>()
for p in Pos:
    if let q = p - 1: best.add(p, p, max(0, best.max(Pos.first, q)))
    for (l, score) in ending[p]: best.add(l, p, score)
print(max(0, best.max()))
