// EGOI '23 P5 (Carnival General): insert each general into a line so that
// both neighbours are in the better half of their ranking. With at least half
// of the line liked, either an end is liked or two adjacent generals are.
// original: corpus/submissions/5678053.cpp
let n: 1..=1000 = read()
domain G = 0..n
var path: List<G> = [G(0)]
for i in 1..n:
    let rank: [0..i]G = read()
    var liked: [G]Bool = false
    for g in rank[..(i + 1) / 2]: liked[g] = true
    let at = if liked[path.first]: 0
        elif liked[path.last]: path.len
        else: first(j + 1 for j in 0..path.len - 1 if liked[path[j]] and liked[path[j + 1]])!
    path = path[..at] + [G(i)] + path[at..]
print(path)
