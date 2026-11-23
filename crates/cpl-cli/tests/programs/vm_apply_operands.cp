// The VM evaluates a variant, its `is` test and payload, a tuple, a cast
// and a comparison from operands in registers, as the tree-walker does.
enum Shape: Dot | Seg(Int) | Box(Int, Int)
struct W(v: Int)
var calls = 0
impl Ord for W:
    fn lt(a, b):
        calls += 1
        a.v < b.v

fn area(s: Shape) -> Int:
    match s:
        Dot => 0
        Seg(n) => n
        Box(w, h) => w * h

var shapes: List<Shape> = []
for i in 0..12:
    shapes.push(if i % 3 == 0: Shape.Dot else: if i % 3 == 1: Shape.Seg(i) else: Shape.Box(i, i + 1))
var total = 0
var dots = 0
for s in shapes:
    total += area(s)
    if s is Dot: dots += 1
print(total, dots)
let pairs = [(i, i * i) for i in 0..5]
var best = (0, 0)
for p in pairs:
    let q = (p.1, p.0)
    if q > best: best = q
print(best.0, best.1, pairs.len)
let ws = [W(3), W(1), W(2)]
var lo = ws[0]
for w in ws:
    if w < lo: lo = w
print(lo.v, calls)
let names = ["b", "a", "c"]
var first = names[0]
for i in 0..names.len:
    if names[i] < first: first = names[i]
print(first, F64(7) / 2.0, yes(I128(7) == I128(names.len + 4)))
