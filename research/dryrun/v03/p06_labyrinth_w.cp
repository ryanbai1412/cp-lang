// p06 with a hand-written BFS on a Deque and steps by hand (rows 6, 27-29).
let (n, m): (1..=1000, 1..=1000) = read()
domain Row = 0..n
domain Col = 0..m
let g = read<[Row, Col]Char>()
fn find(ch: Char) -> (Row, Col) = [(r, c) for r in Row for c in Col if g[r, c] == ch][0]
let (s, t) = (find('A'), find('B'))
var came: [Row, Col]?Dir4 = None
var seen: [Row, Col]Bool = false
var queue = Deque<(Row, Col)>.new()
seen[s] = true
queue.push(s)
while let p = queue.pop_front():
    for d in Dir4.all():
        let (dr, dc) = d.delta
        let (r, c) = (Int(p.0) + dr, Int(p.1) + dc)
        if 0 <= r < n and 0 <= c < m:
            let q = (Row(r), Col(c))
            if g[q] != '#' and not seen[q]:
                seen[q] = true
                came[q] = d
                queue.push(q)
if seen[t]:
    var moves: List<Str> = []
    var p = t
    while p != s:
        let d = came[p]!
        moves.push("{"URDL"[d.k]}")
        let (dr, dc) = d.delta
        p = (Row(Int(p.0) - dr), Col(Int(p.1) - dc))
    print("YES")
    print(moves.len)
    print("".join([moves[i] for i in (0..moves.len).rev]))
else:
    print("NO")
