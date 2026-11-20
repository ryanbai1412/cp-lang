// NOIP 2005 Crossing the River: a frog at 0 hops along a bridge of length
// len, each hop between s and t long, until it is at or past len. Some
// points hold stones. Print the fewest stones it must land on.
// len is up to 10^9 but there are at most 100 stones. 2520 is a multiple of
// every hop length, so a gap between stones can lose any multiple of 2520
// beyond its first: what can be reached past it does not change.
// original: corpus/submissions/5156196.cpp
const CYCLE = 2520
let len: 1..=1_000_000_000 = read()
let s: 1..=10 = read()
let t: s..=10 = read()
let m: 1..=100 = read()
let stones: [m]1..len = read()

var spot: List<Int> = []                    // the stones, moved closer together
var (before, shrunk) = (0, 0)
for x in sorted(stones):
    let gap = x - before
    shrunk += if gap > CYCLE: gap % CYCLE + CYCLE else: gap
    spot.push(shrunk)
    before = x
let end = shrunk + 1                        // past the last stone nothing matters

var stone: [0..end]Bool = false
for x in spot: stone[x] = true
// fewest[p]: stones landed on from p onwards, p included
var fewest: [0..end + t]Int = 0
for p in (0..end).rev:
    fewest[p] = min(fewest[p + hop] for hop in s..=t)! + (if stone[p]: 1 else: 0)
print(fewest[0])
