// JOI 2016 Open Skyscraper: count the orders of n buildings in which
// neighbouring heights differ by at most `limit` in total.
// original: corpus/submissions/4226622.cpp
const P = 1000000007
type Z = Mod<P>
let n: 1..=100 = read()
let limit: 1..=1000 = read()
let heights: [n]1..=1000 = read()

// Buildings go in from the shortest up, forming groups that will later be
// joined into the final row. ways[g, c, e]: g groups, e of them already fixed
// at an end of the row, and c paid so far, counting every open side of a group
// as if its future neighbour were as tall as the building just placed.
type Ways = [0..=n, 0..=limit, 0..=2]Z
var ways: Ways = 0
ways[0, 0, 0] = 1
var level = min(heights)
for h in sorted(heights):
    var next: Ways = 0
    for g in 0..n:
        for c in 0..=limit:
            for e in 0..=2:
                let w = ways[g, c, e]
                let sides = 2 * g - e  // open sides, all rising to h
                let paid = c + (h - level) * sides
                if w != 0 and paid <= limit:
                    next[g + 1, paid, e] += w * (g + 1 - e)            // a group of its own
                    if e < 2: next[g + 1, paid, e + 1] += w * (2 - e)  // ... at an end
                    if g >= 1:
                        next[g, paid, e] += w * sides                  // beside one group
                        if e < 2: next[g, paid, e + 1] += w * (2 - e)  // ... at an end
                    if g >= 2: next[g - 1, paid, e] += w * (g - 1)     // between two groups
    ways = next
    level = h
print(if n == 1: Z.one else: sum(ways[1, c, 2] for c in 0..=limit))
