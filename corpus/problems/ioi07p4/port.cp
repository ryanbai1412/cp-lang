// Source: IOI 2007, Miners, https://dmoj.ca/problem/ioi07p4
// limits: 750 ms, 16 MB
// Problem: N <= 1e5 food shipments, each M, F or B, in order; each goes to one of two mines.
//   A mine digs 1, 2 or 3 coal per shipment: the number of distinct foods among it and that mine's
//   previous two. Print the most coal both mines can dig in total.
// Solution: only each mine's last two foods matter, so going from the last shipment back,
//   rest[x, y] (each mine's last two foods as one of 16 states) is the most the shipments from
//   here on dig. Each shipment goes to mine one or two, adding coal[x, new] and moving that mine
//   to after[x, new]. O(16^2 N).
// original: corpus/submissions/5033986.cpp
// Bounds: n: 1..=1e5
let n: 1..=1e5 = read()
domain Ship = 1..=n

let food: [Ship]Char = read()

// A mine remembers its last two shipments as older * 4 + old, 0 standing for none.
domain Mine = 0..16
let after: [Mine, 1..=3]Mine = [Mine, 1..=3] (m, f) => Mine(m.ord % 4 * 4 + f)
let coal: [Mine, 1..=3]Int = [Mine, 1..=3] (m, f) =>
    sum(1 for g in 1..=3 if g == m.ord / 4 or g == m.ord % 4 or g == f)

// rest[x, y]: the most coal from the ships still to come, the mines remembering x and y
var rest: [Mine, Mine]Int = 0
for s in Ship.rev:
    let f = if food[s] == 'M': 1 elif food[s] == 'F': 2 else: 3
    rest = [Mine, Mine] (x, y) =>
        max(coal[x, f] + rest[after[x, f], y], coal[y, f] + rest[x, after[y, f]])
print(rest[Mine(0), Mine(0)])
