// Source: CCO 2004 Stage 2 Day 2 P1 (Return of Space Turtle), https://dmoj.ca/problem/cco04p4
// limits: 1000 ms, 32 MB
// Problem: two points each loop forever around a closed lattice orbit, given as a start and at
//   most 100 moves of |d| <= 100 units along X or Y, one unit per minute. Print the least
//   Euclidean distance between them over all whole minutes, to 2 decimals.
// Solution: list each orbit's position at every minute of one lap (a lap of length 0 stays
//   put); after the lcm of
//   the two lap lengths (at most about 5e7, both are even and at most 1e4) both are back at
//   their starts together, so take the least squared distance over that many minutes and print
//   its root: O(lcm).
// Bounds: n: 0..=100; d: -100..=100
enum Axis:
    @tag("X") X
    @tag("Y") Y
type Pt = (x: Int, y: Int)

// Where the point is at each minute of one lap; the lap ends back at the start.
fn read_orbit() -> List<Pt>:
    var at: Pt = read()
    let n: 0..=100 = read()
    var orbit: List<Pt> = []
    for _ in 0..n:
        let d: -100..=100 = read()
        let axis: Axis = read()
        let unit = if d > 0: 1 else: -1
        for _ in 0..abs(d):
            orbit.push(at)
            at = match axis:
                X => (at.x + unit, at.y)
                Y => (at.x, at.y + unit)
    if orbit.is_empty: orbit.push(at)
    orbit

fn dist2(p: Pt, q: Pt) -> Int = (p.x - q.x) * (p.x - q.x) + (p.y - q.y) * (p.y - q.y)

let a = read_orbit()
let b = read_orbit()
let closest = min(dist2(a[t % a.len], b[t % b.len]) for t in 0..lcm(a.len, b.len))!
print(sqrt(F64(closest)), precision: 2)
