// ioi09p6 Mecho: the longest wait at the honey after which the bear still gets home. Bees spread
// one cell a minute from every hive (BFS); the bear makes up to s steps a minute, so after a wait
// of w he may enter a cell at step k only while w + k / s is before the bees arrive. Waiting
// longer never helps, so the answer is the last w that escapes.
let (n, s) = read<(1..=800, 1..=1_000)>()
let g = read<[n, n]Char>()
let cell = ch => g.keys.find(p => g[p] == ch)!
let (honey, home) = (cell('M'), cell('D'))
let bees = bfs(p => p.adj4.filter(q => g[q] in "GM"), from: g.keys.filter(p => g[p] == 'H'))
fn escapes(w: Int) -> Bool:
    let safe = (p, k) => (bees.dist[p] ?? inf) > w + k / s
    let walk = bfs(p => p.adj4.filter(q => g[q] in "GD"), from: [honey], until: home, enter: safe)
    safe(honey, 0) and walk.dist[home] != none
print((0..n * n).last_true(escapes) ?? -1)
