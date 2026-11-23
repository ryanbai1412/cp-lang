// p06 with a hand-written BFS on a Deque and steps by hand, each move kept as its index in URDL.
const DR = [-1, 0, 1, 0]
const DC = [0, 1, 0, -1]
let (n, m): (1..=1000, 1..=1000) = read()
domain Row = 0..n
domain Col = 0..m
let g = read<[Row, Col]Char>()
fn find(ch: Char) -> (Row, Col) = [(r, c) for r in Row for c in Col if g[r, c] == ch][0]
let (s, t) = (find('A'), find('B'))
var came: [Row, Col]0..4 = 0
var seen: [Row, Col]Bool = false
var queue = Deque<(Row, Col)>.new()
seen[s] = true
queue.push(s)
while let p = queue.pop_front():
    for k in 0..4:
        let (dr, dc) = (DR[k], DC[k])
        let (r, c) = (Int(p.0) + dr, Int(p.1) + dc)
        if 0 <= r < n and 0 <= c < m:
            let q = (Row(r), Col(c))
            if g[q] != '#' and not seen[q]:
                seen[q] = true
                came[q] = k
                queue.push(q)
if seen[t]:
    var moves: List<Str> = []
    var p = t
    while p != s:
        let k = came[p]
        moves.push("{"URDL"[k]}")
        let (dr, dc) = (DR[k], DC[k])
        p = (Row(Int(p.0) - dr), Col(Int(p.1) - dc))
    print("YES")
    print(moves.len)
    print("".join([moves[i] for i in (0..moves.len).rev]))
else:
    print("NO")
