// Writes through `mut` views of top-level and local lists and of grid
// lines (sort, fill, reverse, swap, cell and compound writes) show in a
// trace as soon as they happen, at top level and in fns.
struct Bag:
    xs: List<Int>

    fn part(self, l: Int, r: Int) -> lends mut List<Int>:
        @law disjoint(l, r, l2, r2) = r < l2 or r2 < l
        mut xs[l..r]

fn shuffle(mut b: Bag):
    b.part(0, 3).sort()
    print(b.xs)
    b.part(3, 6)[1] = 40
    print(b.xs)
    reverse(mut b.part(3, 6))
fn local() -> List<Int>:
    var b = [5, 4, 3, 2, 1]
    b[1..4].sort()
    let v = mut b[0..2]
    v[0] = 7
    v[1] += 3
    v.swap(0, 1)
    v.reverse()
    b
fn on_global():
    g[1..4].sort()
    print(g)
    g[4..6].fill(2)
    print(g)
    reverse(mut g[0..3])
var g = [9, 8, 7, 6, 5, 4, 3]
var b = Bag([6, 5, 4, 3, 2, 1])
domain R = 0..3
domain C = 0..3
var m: [R, C]Int = [R, C] (r, c) => 3 * Int(r) + Int(c)
fn edit():
    reverse(mut m.cols[C(0)])
    print(m.cols[C(0)])
    m.rows[R(1)].sort()
shuffle(mut b)
print(b.xs)
b.part(0, 2).sort()
print(b.xs)
reverse(mut b.part(1, 4))
print(b.xs)
on_global()
print(g)
g[1..5].sort()
print(g)
g[2..4].fill(30)
print(g)
g[0..2].reverse()
print(g)
swap(mut g[0..1], mut g[5..6])
print(g)
print(local())
m.rows[R(1)].reverse()
print(m.rows[R(1)])
m.cols[C(2)].sort(by: x => -x)
print(m.cols[C(2)])
edit()
for r in R: print(m.rows[r])
var h = [9, 8, 7, 6, 5, 4, 3]
fn cells_h():
    let v = mut h[1..4]
    v[0] = 11
    v[1] += 1
    v.reverse()
    print(v.len)
fn deep_h():
    cells_h()
    h[0..2].sort()
cells_h()
print(h)
h[1..3].reverse()
print(h)
deep_h()
print(h)
let w = mut h[3..6]
w[0] = 0
w[1] -= 4
print(w.len)
print(h)
