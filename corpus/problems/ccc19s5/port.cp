// CCC 2019 S5 Triangle: The Data Structure: sum, over every sub-triangle of
// side k in a number triangle, its largest entry. best[i, j] is the largest
// entry of the triangle of the current side with apex (i, j); three of them
// cover a triangle up to one and a half times as big, so the side grows
// geometrically.
let n: 1..=3000 = read()
let k: 1..=n = read()

var best: [0..n, 0..n]Int = 0
for i in 0..n:
    for j in 0..=i: best[i, j] = read()

var side = 1
while side < k:
    let grown = min(k, (side * 3 + 1) / 2)
    let step = grown - side
    for i in 0..=n - grown:
        for j in 0..=i: best[i, j] = max(best[i, j], best[i + step, j], best[i + step, j + step])
    side = grown
print(sum(best[i, j] for i in 0..=n - k for j in 0..=i))
