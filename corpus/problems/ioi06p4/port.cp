// IOI 2006 The Valley of Mexico: n cities stand on a circle and some pairs
// trade. Find a route through every city once, along trading pairs, that
// never crosses itself.
// original: corpus/submissions/5032736.cpp
let n: 1..=1000 = read()
let m: 0..=1e5 = read()
domain City = 1..=n
let deals: [m](a: City, b: City) = read()

var trade: [City, City]Bool = false
for (a, b) in deals:
    trade[a, b] = true
    trade[b, a] = true

fn cw(c: City) -> City: City.at(c + 1) ?? City.first!
fn ccw(c: City) -> City: City.at(c - 1) ?? City.last!

// A route that never crosses itself always covers an arc of the circle, and
// grows it by one city at either end. finish(l, r, at_r): the arc from l
// clockwise to r is visited and the route stands at l or r. Where does it go
// next? `none` if it is stuck.
@memo
fn finish(l: City, r: City, at_r: Bool) -> ?City:
    let here = if at_r: r else: l
    let right = cw(r)
    let left = ccw(l)
    if right == l: return here  // everything is visited
    if trade[here, right] and finish(l, right, true) != none: return right
    if trade[here, left] and finish(left, r, false) != none: return left
    None

fn route() -> ?List<City>:
    for start in City:
        if finish(start, start, false) != None:
            var path = [start]
            var l = start
            var r = start
            var at_r = false
            while cw(r) != l:
                let to = finish(l, r, at_r)!
                at_r = to == cw(r)
                if at_r: r = to else: l = to
                path.push(to)
            return path
    None

if let path = route():
    for c in path: print(c)
else: print(-1)
