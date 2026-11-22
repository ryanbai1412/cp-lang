// noi02p1 Galactic Heroes: ships 1..=30000 start as their own columns. "M a b" moves a's column
// behind b's; "C a b" asks how many ships stand between a and b, or -1 in different columns. A
// potential DSU keeps each ship's place relative to its column: joining puts a's head at
// distance size(b) behind b's head.
domain V = 1..=30_000
var dsu = PotentialDsu<V>()
var head = [V] v => v
for _ in 0..read<1..=500_000>():
    let (op, a, b) = read<(Char, V, V)>()
    if op == 'M':
        let (ra, rb) = (dsu.find(a).root, dsu.find(b).root)
        let (ha, hb, behind) = (head[ra], head[rb], dsu.size(b))
        dsu.unite(hb, ha, behind)
        head[dsu.find(a).root] = hb
    else:
        print(if dsu.diff(a, b) is some(d): abs(d) - 1 else: -1)
