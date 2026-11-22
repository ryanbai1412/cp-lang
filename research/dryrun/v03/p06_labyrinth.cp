// CSES 1193 Labyrinth: shortest A -> B path through '.', printed as URDL moves.
let (n, m): (1..=1000, 1..=1000) = read()
domain Row = 0..n
domain Col = 0..m
let g = read<[Row, Col]Char>()
fn find(ch: Char) -> (Row, Col) = [(r, c) for r in Row for c in Col if g[r, c] == ch][0]
let p = bfs(c => c.adj4.filter(q => g[q] != '#'), from: [find('A')])
if let path = p.path(find('B')):
    print(yes(true))
    print(path.len - 1)
    var moves = ""
    for i in 1..path.len:
        let (a, b) = (path[i - 1], path[i])
        moves += if b.0 < a.0: 'U' elif b.1 > a.1: 'R' elif b.0 > a.0: 'D' else: 'L'
    print(moves)
else:
    print(yes(false))
