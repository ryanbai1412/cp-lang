// A call statement whose `mut` args are places the vm walks to: two
// parts lent by one method, an optional field, and more than eight ints.
let n: 2..=9 = read()
domain V = 1..=n
struct Bag:
    cnt: [V]Int
    best: ?Int

    fn at(self, i: V) -> lends mut Int:
        law disjoint(i, j) = i != j
        lend mut cnt[i]
fn move(mut from: Int, mut to: Int):
    to += from
    from = 0
fn relax(mut best: ?Int, v: Int):
    if best == None or v < best!: best = Some(v)
fn spread(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int, g: Int, h: Int, mut total: Int):
    total += a + b + c + d + e + f + g + h
let zero: [V]Int = 0
var b = Bag(zero, None)
for k in V: b.cnt[k] = Int(k) * 10
let (i, j) = (V(1), V(n))
if i != j:
    move(mut b.at(i), mut b.at(j))
for v in [7, 3, 5]: relax(mut b.best, v)
spread(1, 2, 3, 4, 5, 6, 7, 8, mut b.cnt[i])
print(b.cnt, b.best ?? -1)
