// An Int or tuple element of a list of lists is a copied value, so storing it
// keeps no lend of its source (R05.2.1); a regression from e5c06ee29.
var out = [[1, 2], [3]]
var pairs = [[(1, 2)], [(3, 4)]]
var frames: List<(Int, Int)> = []
for u in 0..2:
    if out[u].len > 0:
        let v = out[u][0]
        frames.push((v, 0))
        let p = pairs[u][0]
        frames.push(p)
        out[u].push(v)
print(frames.len, out[0].len, out[1].len)
