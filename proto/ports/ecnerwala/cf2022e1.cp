@!multitest
type Z = Mod<1_000_000_007>
let (n, m, k, _): (2..=100_000, 2..=100_000, 0..=100_000, 0..=0) = read()
domain Line = 1..=n + m
let cells: [k](r: 1..=n, c: 1..=m, v: 0..(1 << 30)) = read()

var up: [Line]Line = [Line] x => x
var size: [Line]Int = 1
var gap: [Line]Int = 0

fn find(x: Line) -> (root: Line, label: Int):
    var (y, label) = (x, 0)
    while up[y] != y:
        label ^= gap[y]
        y = up[y]
    (root: y, label: label)

fn unite(a: Line, b: Line, v: Int) -> Bool:
    let (ra, la) = find(a)
    let (rb, lb) = find(b)
    if ra == rb: return la ^ lb == v
    let (big, small) = if size[ra] >= size[rb]: (ra, rb) else: (rb, ra)
    up[small] = big
    gap[small] = la ^ lb ^ v
    size[big] += size[small]
    true

let consistent = cells.all(cell => unite(Line(cell.r), Line(n + cell.c), cell.v))
let parts = sum(Int(up[x] == x) for x in Line)
print(if consistent: Z(2) ** (30 * (parts - 1)) else: Z(0))
