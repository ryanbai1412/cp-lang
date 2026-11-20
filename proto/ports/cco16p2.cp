// cp: 0.1 23c1c1b+dirty
// CCO 2016 Splitting Hares: weighted points in the plane; pick a line and take
// everything on one side of it, plus what you like of the points on the line.
// Maximise the weight taken. Some best line passes through a point, so turn a
// line around every point in turn.
let n: 2..=4000 = read()
domain Hare = 1..=n
type Vec = (x: Int, y: Int)
type Spot = (x: -1e6..=1e6, y: -1e6..=1e6, w: -1e5..=1e5)

fn crowded() -> [Hare]Spot:             // small grid, mixed signs: many collinear triples
    [Hare] h => (rand(-3, 3), rand(-3, 3), rand(-9, 9))

let hares: [Hare]Spot = read(gen: crowded) where distinct([(h.x, h.y) for h in hares])

fn cross(a: Vec, b: Vec) -> Int: a.x * b.y - a.y * b.x
fn upper(v: Vec) -> Bool: v.y > 0 or (v.y == 0 and v.x < 0)

// A hare seen from the pivot: the line through both, as a direction in the
// upper half plane, and whether the hare is on that side of the pivot.
type Seen = (dir: Vec, ahead: Bool, w: Int)

fn best_through(pivot: Hare) -> Int:
    let p = hares[pivot]
    var seen: List<Seen> = []
    for h in Hare:
        if h != pivot:
            let v: Vec = (hares[h].x - p.x, hares[h].y - p.y)
            seen.push(if upper(v): (v, true, hares[h].w) else: ((-v.x, -v.y), false, hares[h].w))
    sort(seen, less: (a, b) => cross(a.dir, b.dir) > 0)

    let everyone = sum(s.w for s in seen)
    var right = sum(s.w for s in seen if not s.ahead)   // weight right of the turning line
    var best = 0
    for line in runs(seen, same: (a, b) => cross(a.dir, b.dir) == 0):
        let ahead = sum(s.w for s in line if s.ahead)
        let behind = sum(s.w for s in line if not s.ahead)
        best max= max(right + ahead + behind, right + ahead, everyone - right, everyone - right - ahead)
        right += ahead - behind
    p.w + best

print(max(0, max(best_through(h) for h in Hare)))      // taking nothing is allowed
