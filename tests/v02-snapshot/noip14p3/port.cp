// Source: NOIP 2014 P3 (Flappy Bird), https://dmoj.ca/problem/noip14p3
// Problem: a bird crosses n <= 10000 columns at heights 1..=m (m <= 1000), from any height.
//   Moving into column x, each of any number of taps lifts it up[x], capped at m; no tap drops
//   it down[x]. Height 0 or hitting one of k pipes (a column and a gap strictly inside low..high)
//   ends the game. Print 1 and the fewest taps to cross, or 0 and the most pipes passed.
// Solution: DP over columns: taps[h] = fewest taps to be at height h, None if unreachable.
//   Taps are an unbounded knapsack: sweep h upward with next[min(h + up, m)] min= min(taps[h],
//   next[h]) + 1, then add the drops, then clear the heights outside the pipe's gap. If a
//   column has no reachable height, print the pipes before it. O(n m).
let (n, m, k): (1..=10000, 1..=1000, 0..=10000) = read()
domain X = 1..=n  // the column being arrived at
domain H = 1..=m
let moves: [X](up: 1..=1000, down: 1..=1000) = read()
let pipes: [0..k](at: X, low: 0..=1000, high: 0..=1000) = read()

var gap: [X]?(low: Int, high: Int) = None  // on arriving at a column, stay strictly inside
for (at, low, high) in pipes: gap[at] = (low, high)

var taps: [H]?Int = 0  // fewest taps to be at each height
for x in X:
    let (up, down) = moves[x]
    var next: [H]?Int = None
    for h in H:  // upward, so several taps chain
        next[H.at(h + up) ?? H.last!] min= min(taps[h], next[h]) + 1
    for h in H:
        if let low = H.at(h - down): next[low] min= taps[h]
    if let (low, high) = gap[x]:
        for h in H if Int(h) <= low or Int(h) >= high: next[h] = None
    if all(t == None for t in next):
        print(0)
        print(count(p.at < x for p in pipes))
        return
    taps = next
print(1)
print([t! for t in taps if t != None].min()!)
