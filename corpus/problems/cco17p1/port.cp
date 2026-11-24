// Bounds: k: 1..=1e7

let k: 1..=1e7 = read()

fn pairs(c: Int) -> Int = c * (c - 1) / 2

var sizes: List<Int> = []
var left = k
while left > 0:
    let c = (2..=5000).last_true(c => pairs(c) <= left)!
    left -= pairs(c)
    sizes.push(c)

// Cycle i uses the vertices after the earlier ones; a bridge joins it to the one before.
var trails: List<(Int, Int)> = []
var placed = 0
for c in sizes:
    let (first, last) = (placed + 1, placed + c)
    if placed > 0: trails.push((placed, first))
    trails.push((first, last))
    for v in first..last: trails.push((v, v + 1))
    placed = last
print(placed, trails.len)
for (u, v) in trails: print(u, v)
