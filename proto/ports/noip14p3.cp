// NOIP 2014 Flappy Bird: the bird crosses n columns. In column x a tap lifts
// it by up[x] (any number of taps, the ceiling m stops it), no tap drops it by
// down[x]; touching the floor or a pipe ends the game. Print the fewest taps
// to get across, or how many pipes it can pass.
let (n, m, k): (1..=10000, 1..=1000, 0..=10000) = read()
domain X = 1..=n                            // the column being arrived at
domain H = 1..=m
let moves: [X](up: 1..=1000, down: 1..=1000) = read()
let pipes: [0..k](at: X, low: 0..=1000, high: 0..=1000) = read()

var gap: [X]?(low: Int, high: Int) = none   // on arriving at a column, stay strictly inside
for (at, low, high) in pipes: gap[at] = (low, high)

var taps: [H]?Int = 0                       // fewest taps to be at each height
for x in X:
    let (up, down) = moves[x]
    var next: [H]?Int = none
    for h in H:                             // upward, so several taps chain
        next[h + up ?? H.last] min= min(taps[h], next[h]) + 1
    for h in H:
        if let low = h - down: next[low] min= taps[h]
    if let (low, high) = gap[x]:
        for h in H if Int(h) <= low or Int(h) >= high: next[h] = none
    if all(t == none for t in next):
        print(0)
        print(count(p.at < x for p in pipes))
        return
    taps = next
print(1)
print(min(t! for t in taps if t != none))
