// Source: IOI 2006 Day 2 (The Valley of Mexico), https://dmoj.ca/problem/ioi06p4
// limits: 1000 ms, 64 MB
// Problem: c <= 1000 cities numbered clockwise round a lake and n agreements (pairs of cities).
//   Print a route that visits every city once, moving only between agreeing pairs along
//   straight segments that never cross; print -1 if there is none.
// Solution: a non-crossing route always covers a contiguous arc and grows it by one city at
//   either end. finish(l, r, at_r), memoised: the arc l..r clockwise is done and the route stands
//   at l or r; it returns the next city (cw(r) or ccw(l)) from which the rest can be finished.
//   Try every start, then follow the choices. O(c^2) states, O(1) each.
// original: corpus/submissions/5032736.cpp
// Bounds: n: 1..=1000; m: 0..=n * (n - 1) / 2; a, b: 1..=n
let n: 1..=1000 = read()
let m: 0..=n * (n - 1) / 2 = read()
domain City = 1..=n
var deal: [City, City]Bool = false  // keyed low city first
for _ in 0..m:
    let (a, b): (City, City) = read()
    deal[min(a, b), max(a, b)] = true
fn trade(x: City, y: City) -> Bool = deal[min(x, y), max(x, y)]

fn cw(c: City) -> City: City.at(c + 1) ?? City.first!
fn ccw(c: City) -> City: City.at(c - 1) ?? City.last!

// A route that never crosses itself always covers an arc of the circle, and
// grows it by one city at either end. finish(l, r, at_r): the arc from l
// clockwise to r is visited and the route stands at l or r. Where does it go
// next? None if it is stuck.
@memo
fn can_finish(l: City, r: City, at_r: Bool) -> Bool:
    let here = if at_r: r else: l
    let right = cw(r)
    let left = ccw(l)
    if right == l: return true  // everything is visited
    (trade(here, right) and can_finish(l, right, true)) or
        (trade(here, left) and can_finish(left, r, false))

fn route() -> ?List<City>:
    for start in City:
        if can_finish(start, start, false):
            var path = [start]
            var l = start
            var r = start
            var at_r = false
            while cw(r) != l:
                let here = if at_r: r else: l
                at_r = trade(here, cw(r)) and can_finish(l, cw(r), true)
                if at_r: r = cw(r) else: l = ccw(l)
                path.push(if at_r: r else: l)
            return path
    None

if let path = route():
    for c in path: print(c)
else: print(-1)
