// Educational DP X, Tower: stack some blocks. Block i weighs w, is worth v and
// carries at most s on top of it. Maximise the value of the tower.
// original: corpus/submissions/4133420.cpp
let n: 1..=1000 = read()
let blocks: [n](w: 1..=1e4, s: 1..=1e4, v: 1..=1e9) = read()

// If a goes right under b, then swapping them is only worse when
// a.s + a.w < b.s + b.w. So some best tower is sorted that way from the top,
// and it is built top down: a knapsack over the weight so far, which never
// passes s + w of the bottom block.
var best: [0..=20000]?Int = none        // best[x]: most value in a tower weighing x
best[0] = 0
for b in sorted(blocks, by: b => b.s + b.w):
    for x in (0..=b.s).rev:
        if let v = best[x]: best[x + b.w] max= v + b.v
print(max(best))
