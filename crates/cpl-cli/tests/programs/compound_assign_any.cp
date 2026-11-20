// `a op= b` is `a = a op b` for every type with `op`: a list grows where it
// is, a struct uses its `+`, and an index that does work is done once.
struct V2(x: Int, y: Int)
impl Add for V2:
    fn add(a, b) = V2(a.x + b.x, a.y + b.y)
var xs = [1, 2]
xs += [3, 4]
var s = "ab"
s += "cd"
var v = V2(1, 2)
v += V2(10, 20)
var grid = [[1], [2]]
grid[0] += [9]
var z: Mod<7> = 3
z *= 5
var f = 1.5
f *= 2.0
print(xs, s, v.x, v.y, grid[0], z, f)
var calls = 0
fn at() -> Int:
    calls += 1
    0
grid[at()] += [7, 8]
var ys = [1]
ys += ys
print(grid, calls, ys)
