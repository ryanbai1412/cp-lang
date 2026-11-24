// expect: ok, output "YES YES"
// A `Mat(.., over:)` the user builds is stored in the ring's flat form at build, as a product's
// result is, so a walk that multiplies it again and again gives the brute loop's sums.
domain V = 0..5
var s = 7
fn next() -> Int:
    s = (s * 1103515245 + 12345) % 2_147_483_648
    s
let d: [V, V]?Int = [V, V] (_, _) => if next() % 3 == 0: None else: Some(next() % 100 - 50)
let step = Mat(d, over: Semiring(max, +))
var walk = Mat<?Int>.filled(1, V.count, None, over: Semiring(max, +))
walk.set(0, 0, Some(0))
var brute: [V]?Int = [V] (v) => if v.ord == 0: Some(0) else: None
var same = true
for _ in 0..20:
    walk = walk * step
    var after: [V]?Int = [V] (_) => None
    for u in V:
        if let x = brute[u]:
            for v in V:
                if let w = d[u, v]:
                    if after[v] == None or x + w > after[v]!: after[v] = Some(x + w)
    brute = after
    for v in V: same = same and walk.get(0, v.ord) == brute[v]
let sq = step * step
var square = true
for u in V:
    for v in V:
        var best: ?Int = None
        for k in V:
            if let x = d[u, k]:
                if let y = d[k, v]:
                    if best == None or x + y > best!: best = Some(x + y)
        square = square and sq.get(u.ord, v.ord) == best
print(if same: "YES" else: "NO", if square: "YES" else: "NO")
