let n: 2..=9 = read()
domain V = 1..=n
struct Bag:
    cnt: [V]Int
    alt: [V]Int

    fn at(self, i: V) lends mut Int:
        law disjoint(i, j) = i != j
        lend mut cnt[i]
    fn other(self, i: V) lends mut Int:
        lend mut cnt[i]
    fn peek(self, i: V) lends Int:
        lend cnt[i]
fn move(mut from: Int, mut to: Int):
    to += from
    from = 0
let zero: [V]Int = 0
var b = Bag(zero, zero)
let i = V(1)
let j = V(2)
fn two(mut xs: [V]Int, mut ys: [V]Int):
    xs[V.first] += 1
    ys[V.first] += 2
two(mut b.cnt, mut b.alt)             // different fields are always disjoint
if i != j:
    move(mut b.at(i), mut b.at(j))
print(b.cnt, b.alt)
