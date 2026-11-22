// IOI 2009 "Mecho": bees spread one cell a minute from the hives; Mecho eats
// for w minutes, then walks up to s cells a minute to home. Largest safe w:
// binary search, with a BFS that only enters cells before the bees do.
enum Cell = 'T' Tree | 'G' Grass | 'M' Mecho | 'D' Home | 'H' Hive
let (n, s) = read<(1..=800, 1..=1000)>()
domain P = ([n], [n])
let field = read<[P]Cell>()
let moves = grid4(P)
let bee = bfs(moves.only(p => field[p] in [Grass, Mecho, Hive]), from: P.filter(p => field[p] == Hive)).dist
let (start, home) = (field.find(Mecho)!, field.find(Home)!)

fn safe(w: Int) -> Bool:
    let walk = bfs(moves.only(p => field[p] != Tree and field[p] != Hive), from: [start],
        enter: (p, steps) => p == home or bee[p].is_none_or(b => w + steps / s < b))
    bee[start].is_none_or(b => w < b) and walk.dist[home].is_some()

print((0..=n * n).last_true(safe) ?? -1)
