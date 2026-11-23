// CF 2045M Mirror Maze: shoot each of the 2(R + C) lasers and keep those whose beam
// meets every mirror. '/' turns (di, dj) into (-dj, -di); '\' into (dj, di).
let (r, c): (1..=200, 1..=200) = read()
domain Row = 1..=r
domain Col = 1..=c
let rows = read<[Row]Str>()
let cell = [Row, Col] (i, j) => rows[i][Int(j) - 1]
let total = sum(1 for i in Row, j in Col if cell[i, j] != '.')

fn hits(i0: Int, j0: Int, di0: Int, dj0: Int) -> Int:
    var seen: [Row, Col]Bool = false
    var cnt = 0
    var (i, j, di, dj) = (i0, j0, di0, dj0)
    while let ri = Row.at(i), let cj = Col.at(j):
        let ch = cell[ri, cj]
        if ch != '.':
            if not seen[ri, cj]:
                seen[ri, cj] = true
                cnt += 1
            (di, dj) = if ch == '/': (-dj, -di) else: (dj, di)
        i += di
        j += dj
    cnt

var out: List<Str> = []
for j in 1..=c:
    if hits(1, j, 1, 0) == total: out.push("N{j}")
    if hits(r, j, -1, 0) == total: out.push("S{j}")
for i in 1..=r:
    if hits(i, c, 0, -1) == total: out.push("E{i}")
    if hits(i, 1, 0, 1) == total: out.push("W{i}")
print(out.len)
if out.len > 0: print(out)
