// cp: 0.1 1f59ac0+dirty
// CCC 2021 S3 Lunch Concert: friend i walks w per metre until within d of
// the concert at c. The total walking time is convex in c, so find where it
// stops falling.
let n: 1..=2e5 = read()
let friends: [n](p: 0..=1e9, w: 1..=1000, d: 0..=1e9) = read()

fn walk(c: Int) -> Int:
    sum(f.w * max(0, abs(f.p - c) - f.d) for f in friends)

domain Spot = 0..=1e9
let best = Spot.first_where(c => walk(c.label + 1) >= walk(c.label))!
print(walk(best.label))
