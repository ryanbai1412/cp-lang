// Count paths through a grid with walls, and flood-fill its regions.
let h: 1..=50 = read()
let w: 1..=50 = read()
domain R = 0..h
domain C = 0..w
let cell: [R]Str = read()
const MOD = 1000000007
var ways: [R, C]Int = 0
for r in R:
    for c in C:
        if cell[r][c.ord] == '#':
            continue
        if r.ord == 0 and c.ord == 0:
            ways[r, c] = 1
            continue
        var here = 0
        if let up = r.prev:
            here += ways[up, c]
        if let left = c.prev:
            here += ways[r, left]
        ways[r, c] = here % MOD
print(ways[R.last!, C.last!])
print(ways)
var region: [R, C]Int = 0
var regions = 0
for r in R:
    for c in C:
        if cell[r][c.ord] == '#' or region[r, c] != 0:
            continue
        regions += 1
        var stack: List<(R, C)> = [(r, c)]
        region[r, c] = regions
        while stack.len > 0:
            let (y, x) = stack.pop()!
            var around: List<(R, C)> = []
            if let a = y.prev:
                around.push((a, x))
            if let a = y.next:
                around.push((a, x))
            if let a = x.prev:
                around.push((y, a))
            if let a = x.next:
                around.push((y, a))
            for (ny, nx) in around:
                if cell[ny][nx.ord] != '#' and region[ny, nx] == 0:
                    region[ny, nx] = regions
                    stack.push((ny, nx))
print(regions)
print(region)
