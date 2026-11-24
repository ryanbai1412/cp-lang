let n: 1..=9 = read()
domain V = 1..=n

// a projection lends part of its receiver: no reference is returned and none can be stored
struct Bag:
    cnt: [V]Int
    total: Int

    fn at(self, i: V) -> lends mut Int:
        @law disjoint(i, j) = i != j
        mut cnt[i]

    fn first(self) -> lends Int = cnt[V.first!]

    fn window(self, l: V, r: V) -> lends mut [V]Int:
        @law disjoint(l, r, l2, r2) = r < l2 or r2 < l
        assert l <= r
        mut cnt[l..=r]

fn add(mut x: Int, d: Int):
    x += d
fn move(mut from: Int, mut to: Int):
    to += from
    from = 0

let zero: [V]Int = 0
var b = Bag(zero, 0)
for v in V: b.at(v) = v.label * 10        // a place: assignment goes through the lend
b.at(V(2)) += 5
add(mut b.at(V(3)), 100)
print(b.cnt, b.first(), b.at(V(2)) + 1)

let i = V(1)
let j = V(3)
if i != j:
    move(mut b.at(i), mut b.at(j))        // ok: the law `i != j` holds here
print(b.cnt)

let moved = b.at(j)                       // a field and a projection of one value may overlap: copy first
add(mut b.total, moved)
print(b.total)

b.window(V(2), V(4)).sort()
let win = b.window(V(1), V(2))
print(win.sum(), win.len)
print(b.cnt)
