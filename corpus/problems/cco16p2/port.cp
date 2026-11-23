// CCO 2016 Splitting Hares: weighted points in the plane, no three on a line.
// Pick a line and take everything on one side of it, plus what you like of
// the points on the line. Maximise the weight taken. Some best line passes
// through a point, so turn a line around every point in turn.
use std.geometry.{arg, turn}

let n: 3..=4000 = read()
domain Hare = 1..=n
type Spot = (x: -1e6..=1e6, y: -1e6..=1e6, w: -1e5..=1e5)

fn pos(s: Spot) -> Pt = Pt(s.x, s.y)
fn upper(v: Pt) -> Bool = v.y > 0 or (v.y == 0 and v.x < 0)

fn crowded() -> [Hare]Spot:  // small grid, mixed signs
    [Hare] h => (rand(-4, 4), rand(-4, 4), rand(-9, 9))

fn general_position(hs: [Hare]Spot) -> Bool:
    all(
        turn(
            pos(hs[a]),
            pos(hs[b]),
            pos(hs[c]),
        ) != 0 for a in Hare for b in Hare for c in Hare if a < b and b < c
    )

let hares: [Hare]Spot = read(gen: crowded)
assert general_position(hares)

// A hare seen from the pivot: the line through both, as a direction in the
// upper half plane, and whether the hare is on that side of the pivot.
type Seen = (dir: Pt, ahead: Bool, w: Int)

fn best_through(pivot: Hare) -> Int:
    let p = hares[pivot]
    var seen: List<Seen> = []
    for h in Hare:
        if h != pivot:
            let v = pos(hares[h]) - pos(p)
            seen.push(if upper(v): (v, true, hares[h].w) else: (-v, false, hares[h].w))
    seen.sort(by: s => arg(s.dir))

    let everyone = sum(s.w for s in seen)
    var right = sum(s.w for s in seen if not s.ahead)  // right of the turning line, or behind the pivot on it
    var best = 0
    for s in seen:
        if not s.ahead: right -= s.w  // now strictly right of the line
        let left = everyone - right - s.w
        best max= max(right, left) + max(s.w, 0)
        if s.ahead: right += s.w
    p.w + best

print(max(0, [best_through(h) for h in Hare].max()!))  // taking nothing is allowed
