// cp: 0.1 1f59ac0+dirty
// CCC 2019 S5 Triangle: The Data Structure: sum, over every sub-triangle of
// side k in a number triangle, its largest entry. best[i][j] is the largest
// entry of the triangle of the current side with apex (i, j); three of them
// cover a triangle up to one and a half times as big, so the side grows
// geometrically.
let n: 1..=3000 = read()
let k: 1..=3000 = read() where k <= n
domain Row = 0..n

var best: [Row]List<Int> = []
for r in Row:
    for c in 0..=r.idx:
        let x: 0..=1e9 = read()
        best[r].push(x)

var side = 1
while side < k:
    let grown = min(k, (side * 3 + 1) / 2)
    let step = grown - side
    for i in Row:
        if let below = i + step and i.idx + grown <= n:
            for j in 0..=i.idx:
                best[i][j] = max(best[i][j], best[below][j], best[below][j + step])
    side = grown
print(sum(sum(b for b in best[i]) for i in Row if i.idx + k <= n))
