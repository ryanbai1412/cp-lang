// Source: CCO 2016 P2 (Splitting Hares), https://dmoj.ca/problem/cco16p2
// Problem: 2 <= n <= 4000 distinct points (|x|, |y| <= 1e6) with weights |w| <= 1e4, three may
//   be collinear. Draw a line and take every point on one side of it, points on the line
//   included; print the largest total weight.
// Solution: shrink and turn a best half-plane until its edge passes through a pivot p and
//   another point. For each pivot, sort the others by the direction of the line through them
//   (folded into the upper half plane) and sweep one direction at a time, keeping the weight
//   strictly right of the line. A slight turn about p moves the line's points ahead of p to one
//   side and those behind to the other, so a side takes p and the points ahead, behind or both.
//   O(n^2 log n).
use std.geometry.{arg}

let n: 2..=4000 = read()
domain Hare = 1..=n
type Spot = (x: -1e6..=1e6, y: -1e6..=1e6, w: -10_000..=10_000)

fn pos(s: Spot) -> Pt = Pt(s.x, s.y)
fn upper(v: Pt) -> Bool = v.y > 0 or (v.y == 0 and v.x < 0)

fn crowded() -> [Hare]Spot:  // small grid, mixed signs, many collinear
    [Hare] h => (rand(-4, 4), rand(-4, 4), rand(-9, 9))

let hares: [Hare]Spot = read(gen: crowded)

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
    var best: ?Int = None
    for line in seen.runs(by: s => arg(s.dir)):
        let ahead = sum(s.w for s in line if s.ahead)
        let behind = sum(s.w for s in line if not s.ahead)
        right -= behind  // now strictly right of the line
        let left = everyone - right - ahead - behind
        best max= max(right, left) + max(ahead, behind, ahead + behind)
        right += ahead
    p.w + best!

print(max(0, [best_through(h) for h in Hare].max()!))  // taking nothing is allowed
