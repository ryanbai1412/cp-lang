// IOI 2004 Hermes: a city of streets along every integer x and y. Starting
// at (0, 0), Hermes must see n lighthouses in order; he sees one when he
// stands on its vertical or horizontal street. Print the least walking.
// After seeing lighthouse i he is on its vertical street at some y, or on
// its horizontal street at some x. For the next one he either keeps his
// free coordinate and walks to the new street, or walks along his street
// until the other coordinate matches.
// original: corpus/submissions/4171535.cpp
let n: 1..=20000 = read()
domain Coord = -1000..=1000
let lights: [n](x: Coord, y: Coord) = read()

// on_x[y]: least walk to stand at (x of the last lighthouse, y); on_y likewise
var on_x: [Coord]?Int = None
var on_y: [Coord]?Int = None
on_x[Coord(0)] = 0
on_y[Coord(0)] = 0
var (px, py) = (Coord(0), Coord(0))
for (x, y) in lights:
    let (dx, dy) = (abs(x.ord - px.ord), abs(y.ord - py.ord))
    // leave a street by walking along it to the new lighthouse's other street
    let turn_to_y = [w + abs(v.ord - y.ord) for v in Coord if let w = on_x[v]].min()!
    let turn_to_x = [w + abs(v.ord - x.ord) for v in Coord if let w = on_y[v]].min()!
    for v in Coord:
        if let w = on_x[v]: on_x[v] = w + dx
        if let w = on_y[v]: on_y[v] = w + dy
    on_y[px] min= turn_to_y
    on_x[py] min= turn_to_x
    px = x
    py = y
print(min(w for street in [on_x, on_y] for v in Coord if let w = street[v])!)
