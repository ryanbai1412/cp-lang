// IOI 2009 "Mecho": bees spread one cell a minute from the hives; Mecho eats
// for w minutes, then walks up to s cells a minute to home. Largest safe w:
// binary search, with a BFS that only enters cells before the bees do.
let (n, s) = read<(1..=800, 1..=1_000)>()
domain I = [n]
domain P = (I, I)
let field = read<[P]Char>()
let at = (c: Char) => P.find(p => field[p] == c)!
let (start, home) = (at('M'), at('D'))
let bee = bfs(p => p.adj4.filter(q => field[q] in "GM"), from: P.filter(p => field[p] == 'H'))

fn safe(w: Int) -> Bool:
    let ahead = (p: P, steps: Int) => bee[p].is_none_or(b => w + steps / s < b)
    let walk = bfs(p => p.adj4.filter(q => field[q] in "GD"), from: [start], until: home, enter: ahead)
    ahead(start, 0) and walk.reached(home)

print((0..n * n).last_true(safe) ?? -1)
