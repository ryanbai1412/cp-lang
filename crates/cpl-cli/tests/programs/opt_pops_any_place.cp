// A `pop`, `pop_first` or `pop_last` gives `?T` from any place that names its
// list, set or map: a local, a global, a field, an entry or an element.
struct Bag(xs: List<Int>)
struct Box(bag: Bag, n: Int)
fn take(mut xs: List<Int>) -> ?Int = xs.pop()
fn run():
    var xs = [1, 2]
    print(xs.pop()!, take(mut xs)!, xs.pop() ?? -1)
    var bags = [Bag([3, 4]), Bag([5])]
    var bx = Box(Bag([6, 7]), 0)
    print(bags[1].xs.pop()!, bags[1].xs.pop() ?? -1, bx.bag.xs.pop()!, bx.bag.xs.len)
    var grid = [[1, 2], [3]]
    var g = Map<Int, List<Int>>()
    g[4] = [8, 9]
    print(grid[0].pop()!, grid[1].pop()!, grid[1].pop() ?? -1, g[4].pop()!, g[4].len)
    var gs = Map<Str, Set<Int>>()
    gs["a"] = Set([5, 1, 9])
    var hs = [Set([4, 2, 6])]
    print(gs["a"].pop_first()!, gs["a"].pop_last()!, hs[0].pop_last()!, hs[0].pop_first()!, hs[0].pop_first() ?? -1)
    domain D = 0..3
    var a: [D]?Int = None
    a[D(1)] = 7
    let i = D(1)
    print(a[i] ?? -1, a[D(0)] ?? -1)
run()
var gbags = [Bag([12, 13])]
var gm = Map<Int, Set<Int>>()
gm[0] = Set([14, 15])
print(gbags[0].xs.pop()!, gm[0].pop_first()!, gm[0].pop_last()!, gm[0].pop_last() ?? -1)
