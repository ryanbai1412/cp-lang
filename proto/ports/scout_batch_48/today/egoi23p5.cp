// EGOI '23 P5 (Carnival General): insert each general into a line so that
// both neighbours are in the better half of their ranking.
// Runnable on proto/cpc today.
// original: corpus/submissions/5678053.cpp
let n: 1..=1000 = read()
domain G = 0..n
var path: List<G> = [G(0)]
for i in 1..n:
    var liked: [G]Bool = false
    for j in 0..i:
        let g: G = read()
        if j < (i + 1) / 2: liked[g] = true
    let me = G(i)
    var at = path.len
    if liked[path[0]]: at = 0
    elif not liked[path[path.len - 1]]:
        for j in (0..path.len - 1).rev:
            if liked[path[j]] and liked[path[j + 1]]: at = j + 1
    var next: List<G> = []
    for j in 0..path.len:
        if j == at: next.push(me)
        next.push(path[j])
    if at == path.len: next.push(me)
    path = next
print(path)
