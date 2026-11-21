// IOI 2016 Aliens: n points of interest on an m by m grid. A photo is a
// square whose corners lie on the main diagonal; take at most k photos so
// that every point is in one, and print the least number of cells
// photographed. (Input as the IOI sample grader gives it.)
// A point (r, c) needs the diagonal span min(r, c)..=max(r, c), and a span
// inside another is free. With the rest sorted, both ends increase, and a
// photo covers a run of them. Charging a fee per photo makes the count
// unconstrained; the fee that brings the best count down to k gives the answer.
// original: corpus/submissions/4730545.cpp
let n: 1..=100_000 = read()
let m: 1..=1_000_000 = read()
let k: 1..=n = read()
let points: [n](r: 0..m, c: 0..m) = read()

var spans: List<(lo: Int, hi: Int)> = []
for (lo, far) in sorted([(min(r, c), -max(r, c)) for (r, c) in points]):
    if spans.len == 0 or -far > spans.last!.hi: spans.push((lo: lo, hi: -far))
let cnt = spans.len

type Plan = (cost: Int, photos: Int)             // ordered by cost, then by photos
type Line = (slope: Int, at0: Int, photos: Int)  // cost = slope * x + at0

fn value(l: Line, x: Int) -> Plan:
    (cost: l.slope * x + l.at0, photos: l.photos)

// the first whole x from which b, the steeper-falling line, is no higher than a
fn overtakes(a: Line, b: Line) -> Int:
    let fall = a.slope - b.slope
    (b.at0 - a.at0 + fall - 1) / fall

// between a and c (slopes falling), b is never the lowest at a whole x
fn useless(a: Line, b: Line, c: Line) -> Bool:
    overtakes(a, c) <= overtakes(a, b)

// cheapest cover when each photo costs `fee` extra, and its photo count
fn cover(fee: Int) -> Plan:
    var hull: List<Line> = []  // live lines are hull[head..]
    var head = 0
    var done: Plan = (cost: 0, photos: 0)  // best plan for the spans so far
    for i in 0..cnt:
        // a photo starting at span i: its square, less the part the last photo shares
        let lo = spans[i].lo
        let shared = if i > 0: max(0, spans[i - 1].hi - lo + 1) else: 0
        let line: Line = (
            slope: -2 * lo,
            at0: done.cost + lo * lo - shared * shared + fee,
            photos: done.photos + 1,
        )
        while hull.len - head >= 2 and useless(hull[hull.len - 2], hull.last!, line): hull.pop()
        hull.push(line)
        let x = spans[i].hi + 1  // the ends only grow, so drop beaten lines
        while hull.len - head >= 2 and not (
            value(hull[head], x) < value(hull[head + 1], x)
        ): head += 1
        let v = value(hull[head], x)
        done = (cost: v.cost + x * x, photos: v.photos)
    done

// a higher fee never means more photos; find the lowest fee that fits k
let fee = bsearch_first(0..m * m, mid => cover(mid).photos <= k) ?? (m * m)
print(cover(fee).cost - k * fee)
