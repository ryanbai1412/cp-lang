// `?Int max=` on every kind of place: `None` orders below every Int.
struct Best:
    var at: ?Int
    var n: Int

var g: ?Int = None
g max= None
g max= Some(-4)
g max= Some(-9)
g max= None
var cells: [3]?Int = None
cells[0] max= Some(5)
cells[0] max= None
cells[2] max= Some(-1)
var xs: List<?Int> = [None, Some(2)]
xs[0] max= Some(-3)
xs[1] max= Some(1)
xs[1] max= Some(6)
var b = Best(None, 0)
b.at max= Some(8)
b.at max= Some(7)
print(g ?? 99, cells[0] ?? 99, cells[1] ?? 99, cells[2] ?? 99)
print(xs[0] ?? 99, xs[1] ?? 99, b.at ?? 99)

fn best_of(ys: List<Int>) -> ?Int:
    var m: ?Int = None
    for y in ys: m max= Some(y)
    m max= None
    m

print(best_of([]) ?? 99, best_of([-7, -2, -5]) ?? 99)
