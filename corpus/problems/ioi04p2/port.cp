// Source: IOI 2004 (Hermes), https://dmoj.ca/problem/ioi04p2
// limits: 1000 ms, 16 MB
// Problem: streets run along every integer x and y. Starting at (0, 0), Hermes must, in order,
//   visit some point on the horizontal or the vertical street of each of n <= 20000 junctions
//   (X_i, Y_i), |X_i|, |Y_i| <= 1000. Print the least total walking distance.
// Solution: after junction i he stands on its vertical street at some y, or its horizontal
//   street at some x, so on_x[y] / on_y[x] (2001 entries each) hold the least walk to each such
//   state. For the next junction he either keeps his free coordinate (every entry grows by |dx|
//   or |dy|) or turns where his street meets the new junction's other street and walks along
//   it. Both are one elementwise pass per array. O(n * 2001).
// original: corpus/submissions/4171535.cpp
// Bounds: n: 1..=20000; lights.x, lights.y: -1000..=1000
let n: 1..=20000 = read()
domain Coord = -1000..=1000
let lights: [n](x: Coord, y: Coord) = read()

// on_x[y]: least walk to stand at (x of the last lighthouse, y); on_y likewise. Walking along
// a street never beats standing where it meets the next one, so each entry grows by the step
// across or turns at that meeting point.
var on_x: [Coord]Int = [Coord] v => abs(v.label)
var on_y: [Coord]Int = [Coord] v => abs(v.label)
var (px, py) = (Coord(0), Coord(0))
for (x, y) in lights:
    let (ay, ax) = (on_y[x], on_x[y])
    let (dx, dy) = (abs(x.ord - px.ord), abs(y.ord - py.ord))
    for v in Coord:
        on_x[v] = min(on_x[v] + dx, ay + abs(v.ord - py.ord))
        on_y[v] = min(on_y[v] + dy, ax + abs(v.ord - px.ord))
    px = x
    py = y
print(min(on_x.min()!, on_y.min()!))
