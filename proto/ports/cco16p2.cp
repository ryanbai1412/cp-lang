// cp: 0.1 0abd108+dirty
// CCO 2016 Splitting Hares: weighted points in the plane, no three on a line.
// Pick a line and take everything on one side of it, plus what you like of
// the points on the line. Maximise the weight taken. Some best line passes
// through a point, so turn a line around every point in turn.
let n: 3..=4000 = read()
domain Hare = 1..=n
type Vec = (x: Int, y: Int)
type Spot = (x: -1e6..=1e6, y: -1e6..=1e6, w: -1e5..=1e5)

fn cross(a: Vec, b: Vec) -> Int: a.x * b.y - a.y * b.x
fn upper(v: Vec) -> Bool: v.y > 0 or (v.y == 0 and v.x < 0)

fn crowded() -> [Hare]Spot:             // small grid, mixed signs
    [Hare] h => (rand(-4, 4), rand(-4, 4), rand(-9, 9))

fn general_position(hs: [Hare]Spot) -> Bool:
    all(cross((hs[b].x - hs[a].x, hs[b].y - hs[a].y), (hs[c].x - hs[a].x, hs[c].y - hs[a].y)) != 0
        for a in Hare for b in Hare for c in Hare if a < b and b < c)

let hares: [Hare]Spot = read(gen: crowded) where general_position(hares)

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
    var right = sum(s.w for s in seen if not s.ahead)   // right of the turning line, or behind the pivot on it
    var best = 0
    for s in seen:
        if not s.ahead: right -= s.w                    // now strictly right of the line
        let left = everyone - right - s.w
        best max= max(right, left) + max(s.w, 0)
        if s.ahead: right += s.w
    p.w + best

print(max(0, max(best_through(h) for h in Hare)))      // taking nothing is allowed
