// `with io(base: 1)` prints a counted position in base 1 inside lists, tuples, records, structs, options, sets and grids too; an explicit-end domain prints its label (R10.2.2).
with io(base: 1)
domain E = 1..=4
struct At: at: V, w: Int
struct Named: at: V
impl Show for Named:
    fn show(n) = "#{n.at.ord}"
domain V = read(1..=5)
let x: V = read()
let l: List<V> = [x, V(0)]
print(x)
print(l)
print(1, l)
print(l, sep: ",")
print((x, x))
print((at: x, n: 7))
let a: [0..2]V = x
print(a)
print(Some(x))
let ps: List<(V, Int)> = [(x, 5), (V(0), 6)]
print(ps)
print("{l} {x}")
let g: [0..2, 0..2]V = x
print(g)
let cube: [0..2, 0..1, 0..2]V = x
print(cube)
print(At(x, 3))
let ats: List<At> = [At(x, 1)]
print(ats)
print(E(2), [E(2)])
print(Set([x, V(0)]))
print(Named(x))
