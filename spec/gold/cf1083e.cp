// Choose rectangles (0,0)-(x, y) maximising union area minus Σ a. Sorted by x, y falls, so
// choosing i after j adds (xᵢ - xⱼ)·yᵢ, and the best previous j is a max of lines at yᵢ.
let n: 1..=1_000_000 = read()
var rects = read<[n](x: 1..=1e9, y: 1..=1e9, a: Int)>()
rects.sort(by: r => r.x)

var lines = LiChao(over: 1..=1e9, keep: max)
lines.add(k: 0, b: 0)   // nothing chosen before
var answer = 0
for r in rects:
    let v = r.x * r.y - r.a + lines[r.y]
    answer max= v
    lines.add(k: -r.x, b: v)
print(answer)
