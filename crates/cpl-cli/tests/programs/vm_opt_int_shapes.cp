// A `?Int` the vm makes with its own ops: a method on `mut self` taking a
// fn value, a swap of a global with a captured local, and a `?Int` read
// through an unwrap, a payload, a map, a list or a coalesce.
struct Bag:
    xs: List<Int>
    fn path(mut self, i: Int, f: (Int) -> ?Int) -> ?Int:
        if i < 0: None else: f(xs.pop()!)

fn pop_big(mut b: Bag, lim: Int) -> ?Int:
    if b.xs.len == 0: return None
    let x = b.xs.pop()!
    if x > lim: x else: None

enum Slot:
    Full(v: ?Int)
    Empty

var g: ?Int = 41
var bag = Bag([1, 9, 3, 12])
let n: Int = read()
print(pop_big(mut bag, n) ?? -1, pop_big(mut bag, n) ?? -1, bag.xs)
var rope = Bag([4, n, 6])
print(rope.path(1, v => Some(v + n)) ?? -1, rope.path(-1, v => Some(v)) ?? -1, rope.xs)
var held: ?Int = n
let peek = () => held ?? 0
swap(mut g, mut held)
print(g ?? -1, held ?? -1, peek())
let deep: ?(?Int) = Some(Some(n * 2))
let s = Full(n + 1)
var m: Map<Int, ?Int> = []
m[n] = n * 3
let rows: List<?Int> = [None, n]
let none_first: ?Int = None
var sum = 0
for i in 0..2:
    let a: ?Int = deep!
    let b: ?Int = match s:
        Full(v) => v
        Empty => None
    let c: ?Int = m[n]
    let d: ?Int = rows[i]
    let e: ?Int = none_first ?? rows[i]
    sum += (a ?? 0) + (b ?? 0) + (c ?? 0) + (d ?? 0) + (e ?? 0)
print(sum)
