// Source: IOI 2007, Miners, https://dmoj.ca/problem/ioi07p4
// Problem: N <= 1e5 food shipments, each M, F or B, in order; each goes to one of two mines.
//   A mine digs 1, 2 or 3 coal per shipment: the number of distinct foods among it and that mine's
//   previous two. Print the most coal both mines can dig in total.
// Solution: only each mine's last two foods matter, so most[a, b, c, d] (0 = none yet, else a
//   food) is the best total so far. Each shipment goes to mine one or two, shifting that mine's
//   pair and adding coal(older, old, new). O(4^4 N).
// original: corpus/submissions/5033986.cpp
let n: 1..=1e5 = read()
domain Ship = 1..=n

fn menu() -> [Ship]Char:
    let kinds: List<Char> = ['M', 'F', 'B']
    [Ship] _ => kinds[rand(0, 2)]  // WISH: "MFB"[rand(0, 2)]

let food: [Ship]Char = read(gen: menu)

// A mine remembers its last two foods: 0 for "nothing yet", else 1..=3.
fn coal(older: Int, old: Int, new: Int) -> Int:
    sum(1 for f in 1..=3 if f == older or f == old or f == new)

// most[a, b, c, d]: the most coal so far when mine one last got a then b and
// mine two last got c then d.
type Memory = [0..=3, 0..=3, 0..=3, 0..=3]?Int
var most: Memory = None
most[0, 0, 0, 0] = 0
for s in Ship:
    let f = if food[s] == 'M': 1 elif food[s] == 'F': 2 else: 3
    var next: Memory = None
    for a in 0..=3:
        for b in 0..=3:
            for c in 0..=3:
                for d in 0..=3:
                    if let dug = most[a, b, c, d]:
                        next[b, f, c, d] max= dug + coal(a, b, f)
                        next[a, b, d, f] max= dug + coal(c, d, f)
    most = next
print(most.max())
