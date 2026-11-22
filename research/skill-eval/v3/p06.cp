let (n, m) = read<(1..=1000, 1..=1000)>()
domain R = 1..=n
domain C = 1..=m
let maze = read<[R, C]Char>()
var start: ?(R, C) = None
var goal: ?(R, C) = None
for r in R:
    for c in C:
        if maze[r, c] == 'A':
            start = Some((r, c))
        if maze[r, c] == 'B':
            goal = Some((r, c))
let a = start!
let b = goal!
let paths = bfs(p => p.adj4.filter(q => maze[q] != '#'), from: [a], until: b)
fn letter(p: (R, C), q: (R, C)) -> Char:
    let d = Dir4.all().first(d => p.step(d) == Some(q))!
    "URDL"[d.k]
if let Some(path) = paths.path(b):
    print("YES")
    print(path.len - 1)
    print([letter(p, q) for (p, q) in path.adjacent], sep: "")
else:
    print("NO")
