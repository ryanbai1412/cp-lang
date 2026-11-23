// NOI 2010 P3, Altitude: an (n+1) x (n+1) lattice of junctions, every street
// segment carrying a flow in each of its two directions. Heights are 0 at the
// top-left and 1 at the bottom-right; choose the rest to minimise the total
// climbing effort. The answer is a minimum cut, found as a shortest walk in
// the dual: a walker threads between junctions from the left/bottom border
// to the top/right border, and each turn pays for the segments it sweeps
// across at the junction it passes.
//
// A state is (junction, direction the walker entered from); junctions one
// step off the lattice are the exits, so coordinates run -1..=n+1 and are
// stored shifted by one.
// Runnable on proto/cpc today.
// original: corpus/submissions/5672180.cpp
let n0: 1..=500 = read()
let n = n0 + 1  // junctions per side
domain X = 0..n
domain Y = 0..n
domain Dir = 0..8      // W, SW, S, SE, E, NE, N, NW
domain Ext = 0..n + 2  // a coordinate + 1
type State = (Ext, Ext, Dir)
const DX = [0, 1, 1, 1, 0, -1, -1, -1]
const DY = [-1, -1, 0, 1, 1, 1, 0, -1]

// Flows leaving each junction towards E, S, W, N, in the statement's order.
let east: [X, 0..n - 1]Int = read()
let south: [0..n - 1, Y]Int = read()
let west: [X, 1..n]Int = read()
let north: [1..n, Y]Int = read()
// The flow leaving junction (x, y) in an axis direction, 0 off the lattice.
fn flow(x: Int, y: Int, dir: Int) -> Int:
    if dir == 0: return if y >= 1: west[X(x), y] else: 0
    if dir == 2: return if x < n - 1: south[x, Y(y)] else: 0
    if dir == 4: return if y < n - 1: east[X(x), y] else: 0
    if x >= 1: north[x, Y(y)] else: 0

var dist: [Ext, Ext, Dir]?Int = None
var pq = MinHeap<(Int, State)>()
fn relax(s: State, d: Int):
    if dist[s.0, s.1, s.2] == None or d < dist[s.0, s.1, s.2]!:
        dist[s.0, s.1, s.2] = d
        pq.push((d, s))

for i in 0..n:
    relax((Ext(i + 1), Ext(1), Dir(0)), 0)  // entering from the west border
    relax((Ext(n), Ext(i + 1), Dir(2)), 0)  // entering from the south border

while let (d, s) = pq.pop():
    if dist[s.0, s.1, s.2] != d: continue
    let x = s.0.idx - 1
    let y = s.1.idx - 1
    if x < 0 or x >= n or y < 0 or y >= n: continue  // an exit
    if x == n - 1 and y == n - 1: continue           // the height-1 corner
    // Turn counter-clockwise from the entry direction, paying for each street swept.
    var paid = 0
    for turn in 1..7:
        let dir = (s.2.idx + turn) % 8
        let nx = x + DX[dir]
        let ny = y + DY[dir]
        if turn > 1 and not (nx == n - 1 and ny == n - 1):
            relax((Ext(nx + 1), Ext(ny + 1), Dir((dir + 4) % 8)), d + paid)
        if dir % 2 == 0: paid += flow(x, y, dir)

var best: ?Int = None
for i in 0..n:
    for exit in [dist[Ext(0), Ext(i + 1), Dir(2)], dist[Ext(i + 1), Ext(n + 1), Dir(0)]]:
        if let v = exit:
            if best == None or v < best!: best = v
print(best!)
