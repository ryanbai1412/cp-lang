// NOI 2016 Grid: given an n x m grid with k blocked (black) cells, find the
// minimum number of additional white cells to block so the remaining white
// cells become disconnected (or -1 if <= 2 white cells remain).
//
// Answer is always in {-1, 0, 1, 2}:
//   - 0 if the white cells are already disconnected, checked by sweeping
//     blocked rows and uniting overlapping white horizontal segments;
//   - -1 if n * m - k <= 2;
//   - 1 if any white cell adjacent (8-way) to a blocked cell or grid border
//     is an articulation point of the radius-2 neighborhood around blocked
//     cells (or when k = 0 and min(n, m) = 1);
//   - 2 otherwise.
// Uses `scout.connectivity.articulation_points` and `std.graph.components`.
// original: corpus/submissions/4798070.cpp
use std.graph.{components}
use scout.connectivity.{articulation_points}

/// Check whether the white cells of an n x m grid minus `blocked` are disconnected.
fn is_disconnected(n: Int, m: Int, blocked: List<(Int, Int)>) -> Bool:
    var by_row: Map<Int, List<Int>> = Map()
    for (r, c) in blocked:
        var cols = by_row.get(r) ?? []
        cols.push(c)
        by_row[r] = cols
    by_row[n] = []
    let rows = sorted([r for (r, _) in by_row])

    var seg_adj = List<List<Int>>()
    fn add_seg() -> Int:
        let id = seg_adj.len
        seg_adj.push([])
        id
    fn link(a: Int, b: Int):
        seg_adj[a].push(b)
        seg_adj[b].push(a)

    var segs = List<(Int, Int, Int)>()
    var prev_r = -1
    if rows[0] > 0:
        segs.push((0, m - 1, add_seg()))

    for r in rows:
        if r - prev_r > 1:
            let id = add_seg()
            for (_, _, sid) in segs: link(sid, id)
            segs = [(0, m - 1, id)]
        if r == n: continue
        var cols = by_row[r].sorted()
        cols.push(m)
        var (l, j) = (0, 0)
        var next_segs = List<(Int, Int, Int)>()
        for c in cols:
            if l < c:
                let id = add_seg()
                while j < segs.len and segs[j].1 < l: j += 1
                while j < segs.len and segs[j].1 < c:
                    link(id, segs[j].2)
                    j += 1
                if j < segs.len and segs[j].0 < c:
                    link(id, segs[j].2)
                next_segs.push((l, c - 1, id))
            l = c + 1
        segs = next_segs
        prev_r = r

    if seg_adj.len <= 1: return false
    domain Seg = 0..seg_adj.len
    let (C, _) = components::<Seg>(u => [(to: Seg(v)) for v in seg_adj[u.to_idx]])
    C.count > 1

/// Check whether blocking one white cell disconnects the white region.
fn has_cut_cell(n: Int, m: Int, blocked: List<(Int, Int)>) -> Bool:
    if blocked.is_empty: return n == 1 or m == 1
    var blk = Set<(Int, Int)>()
    for p in blocked: blk.add(p)

    let radius = if min(n, m) == 1: 3 else: 2
    var near = Set<(Int, Int)>()
    for (x, y) in blocked:
        for dx in -radius..=radius:
            for dy in -radius..=radius:
                let (nx, ny) = (x + dx, y + dy)
                if 0 <= nx and nx < n and 0 <= ny and ny < m and not blk.contains((nx, ny)):
                    near.add((nx, ny))

    let cells = sorted([p for p in near])
    if cells.is_empty: return false
    var id_of = Map<(Int, Int), Int>()
    for i in 0..cells.len: id_of[cells[i]] = i

    domain Cell = 0..cells.len
    var adj: [Cell]List<Cell> = [Cell] _ => []
    for i in Cell:
        let (x, y) = cells[i.to_idx]
        for (nx, ny) in [(x + 1, y), (x, y + 1)]:
            if let j = id_of.get((nx, ny)):
                adj[i].push(Cell(j))
                adj[Cell(j)].push(i)

    let cut = articulation_points::<Cell>(u => [(to: v) for v in adj[u]])
    for i in Cell:
        if not cut[i]: continue
        let (x, y) = cells[i.to_idx]
        for dx in -1..=1:
            for dy in -1..=1:
                if dx == 0 and dy == 0: continue
                let (nx, ny) = (x + dx, y + dy)
                if nx < 0 or nx >= n or ny < 0 or ny >= m or blk.contains((nx, ny)):
                    return true
    false

let t: Int = read()
for _ in 0..t:
    let (n, m, k): (Int, Int, Int) = read()
    var blocked = List<(Int, Int)>()
    for _ in 0..k:
        let (x, y): (Int, Int) = read()
        blocked.push((x - 1, y - 1))
    if is_disconnected(n, m, blocked):
        print(0)
    elif n * m - k <= 2:
        print(-1)
    elif has_cut_cell(n, m, blocked):
        print(1)
    else:
        print(2)
