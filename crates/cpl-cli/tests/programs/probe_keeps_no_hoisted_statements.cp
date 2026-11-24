// A generic struct's static fn learns its type arguments by lowering the argument once
// to see its type; the loop that probe hoists must not run (it made a recursive DFS
// over `Slice` children exponential).
var calls = 0
fn f(x: Int) -> Int:
    calls += 1
    x
struct Box<K>(pub n: Int):
    pub fn of(xs: List<K>) -> Box<K> = Box<K>(xs.len)
let a = [10, 20, 30]
print(Box.of(a.drop(0).map(x => f(x)).collect()).n, calls)
calls = 0
print(SortedBag.of(List(a.take(3).drop(1).map(x => f(x)))).len, calls)
calls = 0
let kids = [v + 1 for v in 0..11]
fn up(v: Int) -> Int:
    calls += 1
    if v == 11: return 0
    SortedBag.of(List(kids.take(v + 1).drop(v).map(c => up(c) + 1))).len
print(up(0), calls)
