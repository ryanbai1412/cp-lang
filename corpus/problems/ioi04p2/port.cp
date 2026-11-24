// Source: IOI 2004 (Hermes), https://dmoj.ca/problem/ioi04p2
// limits: 1000 ms, 16 MB
// Problem: streets run along every integer x and y. Starting at (0, 0), Hermes must, in order,
//   visit some point on the horizontal or the vertical street of each of n <= 20000 junctions
//   (X_i, Y_i), |X_i|, |Y_i| <= 1000. Print the least total walking distance.
// Solution: after junction i he stands on its vertical street at some y, or its horizontal
//   street at some x, so on_x[y] / on_y[x] (2001 entries each) hold the least walk to each such
//   state. For the next junction he either keeps his free coordinate (every entry grows by |dx|
//   or |dy|) or walks along his street to the new junction's other street, landing on it at the
//   old street's coordinate (one min over the array). O(n * 2001).
// original: corpus/submissions/4171535.cpp
// Bounds: n: 1..=20000; lights.x, lights.y: -1000..=1000
let n: 1..=20000 = read()
domain Coord = -1000..=1000
let lights: [n](x: Coord, y: Coord) = read()

// on_x[y] + grown_x: least walk to stand at (x of the last lighthouse, y); on_y likewise. Keeping
// a free coordinate grows every entry alike, so that growth is one offset per street.
var on_x: [Coord]?Int = None
var on_y: [Coord]?Int = None
on_x[Coord(0)] = 0
on_y[Coord(0)] = 0
var (grown_x, grown_y) = (0, 0)
var (px, py) = (Coord(0), Coord(0))
for (x, y) in lights:
    // leave a street by walking along it to the new lighthouse's other street
    let turn_to_y = grown_x + [w + abs(v.ord - y.ord) for v in Coord if let w = on_x[v]].min()!
    let turn_to_x = grown_y + [w + abs(v.ord - x.ord) for v in Coord if let w = on_y[v]].min()!
    grown_x += abs(x.ord - px.ord)
    grown_y += abs(y.ord - py.ord)
    on_y[px] min= turn_to_y - grown_y
    on_x[py] min= turn_to_x - grown_x
    px = x
    py = y
print(min(grown_x + [w for v in Coord if let w = on_x[v]].min()!,
          grown_y + [w for v in Coord if let w = on_y[v]].min()!))
