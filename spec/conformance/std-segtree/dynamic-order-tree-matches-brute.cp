// expect: ok, output "2 1 2\n9 2 0\nYES"
use std.segtree.{DynamicOrderTree}
domain D = 0..3
let a: [D]Int = [3, 1, 4]
var t = DynamicOrderTree(a)
print(t.count_lt(D(0)..=D(2), 4), t.count_lt(D(1)..=D(2), 4), t.count_in(D(0)..=D(2), 1, 3))
t.set(D(2), 9)
print(t.get(D(2)), t.count_lt(D(0)..=D(2), 5), t.count_in(D(0)..=D(2), 4, 2))
use std.rand.Rng
domain W = 0..30
var rng = Rng(5)
fn r(mut g: Rng, m: Int) -> Int = Int(g.next() % U64(m))
var b: [W]Int = 0
for i in W: b[i] = r(mut rng, 20)
var u = DynamicOrderTree(b)
var ref = [b[i] for i in W]
var ok = true
for _ in 0..3000:
    let op = r(mut rng, 3)
    let l = r(mut rng, 30)
    let h = l + r(mut rng, 30 - l)
    if op == 0:
        let x = r(mut rng, 20)
        u.set(W(l), x)
        ref[l] = x
    elif op == 1:
        let x = r(mut rng, 21)
        var c = 0
        for i in l..=h: if ref[i] < x: c += 1
        if u.count_lt(W(l)..=W(h), x) != c: ok = false
    else:
        let x = r(mut rng, 21)
        let y = r(mut rng, 21)
        var c = 0
        for i in l..=h: if x <= ref[i] and ref[i] <= y: c += 1
        if u.count_in(W(l)..=W(h), x, y) != c: ok = false
        if u.get(W(l)) != ref[l]: ok = false
print(yes(ok))
