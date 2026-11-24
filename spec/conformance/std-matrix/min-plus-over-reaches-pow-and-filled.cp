// expect: ok, output "10 10 YES\nYES"
// A non-default `over:` carries into `*`, `**` and `filled`: min-plus walks, None as no path.
domain V = 0..3
let d: [V, V]?Int = [V, V] (i, j) => if i == j: Some(0) elif j.ord == i.ord + 1: Some(5) else: None
let m = Mat(d, over: Semiring(min, +))
let p = m ** 2
print(p.get(0, 2)!, (m * m).get(0, 2)!, yes(p.get(2, 0) == None))
let f = Mat<?Int>.filled(2, 2, None, over: Semiring(min, +))
print(yes(f.get(0, 1) == None))
