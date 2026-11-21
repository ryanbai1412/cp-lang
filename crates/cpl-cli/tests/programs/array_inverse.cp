// Partial-injection and total-bijection `a.inverse()`.
domain Pos = 1..=5
domain Val = 0..4
enum Dir: N, E, S, W

// Partial injection into a named domain: `[Pos]?Val -> [Val]?Pos`
var a: [Pos]?Val = none
a[Pos(1)] = Val(2)
a[Pos(4)] = Val(0)
let inv_a = a.inverse()
print(inv_a[Val(0)]!.label, inv_a[Val(1)] == none, inv_a[Val(2)]!.label, inv_a[Val(3)] == none)

// Partial injection into a bounded Int range (1-based and 0-based):
let n = 5
domain P = 1..=n
var b: [P]?(1..=n) = none
b[P(2)] = 5
b[P(5)] = 1
let inv_b = b.inverse()
print(inv_b[1]!.label, inv_b[3] == none, inv_b[5]!.label)

var c: [Pos]?(0..Pos.count) = none
c[Pos(3)] = 0
c[Pos(5)] = 4
let inv_c = c.inverse()
print(inv_c[0]!.label, inv_c[2] == none, inv_c[4]!.label)

// Partial injection into a fieldless enum: `[Pos]?Dir -> [Dir]?Pos`
var d: [Pos]?Dir = none
d[Pos(2)] = W
d[Pos(5)] = N
let inv_d = d.inverse()
print(inv_d[N]!.label, inv_d[E] == none, inv_d[W]!.label)

// Total bijection `[I]V -> [V]I` without importing `std.seq.inverse`:
domain I = 0..3
domain V = 1..=3
let perm: [I]V = [V(3), V(1), V(2)]
let inv_perm = perm.inverse()
print(inv_perm[V(1)].ord, inv_perm[V(2)].ord, inv_perm[V(3)].ord)
