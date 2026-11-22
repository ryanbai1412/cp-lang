// Interp / sanitized C++ parity for append-only Prefix.filling across 1D, 2D, and List
// in both Forward and Reverse fill orders.
domain D = 1..=5
let a: [D]Int = [3, -1, 4, 1, 5]

var fwd1 = Prefix<D, Int>.filling()
var out1: List<Int> = []
for i in D:
    fwd1.set(i, a[i])
    out1.push(fwd1.fold(D.first!..=i))
    out1.push(fwd1.prefix(i.r))
print(out1)

var rev1 = Prefix<D, Int>.filling(order: FillOrder.Reverse)
var out2: List<Int> = []
for i in D.rev:
    rev1.set(i, a[i])
    out2.push(rev1.fold(i..=D.last!))
print(out2)
print(rev1.prefix(D(3).r))

domain R = 0..3
domain C = 1..=4
let g = [R, C] (r, c) => (r.ord + 1) * 5 - c.ord * 2

var fwd2 = Prefix<R, C, Int>.filling()
for r in R:
    for c in C:
        fwd2.set(r, c, g[r, c])
print(fwd2.fold(R(0)..=R(2), C(2)..=C(4)))
print(fwd2.prefix(R(1).r, C(3).r))

var rev2 = Prefix<R, C, Int>.filling(order: FillOrder.Reverse)
var diag: List<Int> = []
for r in R.rev:
    for c in C.rev:
        let below = rev2.fold(r.r..R.last!.r, c.r..C.last!.r)
        rev2.set(r, c, g[r, c] + below)
        if r.ord == c.ord: diag.push(rev2.fold(r..=R.last!, c..=C.last!))
print(diag)

var fl = Prefix<Int>.filling(4)
for i in 0..4:
    let v = if i == 0: 1 else: fl.fold(0..i) + 1
    fl.set(i, v)
print(fl.fold(0..4))
print(fl.fold(2..4))

var rl = Prefix<Int>.filling(4, order: FillOrder.Reverse)
for i in (0..4).rev:
    let v = if i == 3: 1 else: rl.fold(i + 1..4) + 1
    rl.set(i, v)
print(rl.fold(0..4))
print(rl.prefix(2))
