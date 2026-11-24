struct Pt:
    x: Int
    y: Int

fn cross(self: Pt, o: Pt) -> Int = self.x * o.y - self.y * o.x

let poly = [Pt(0, 0), Pt(4, 0), Pt(4, 3), Pt(0, 3)]
var area = 0
for (u, v) in poly.cyclic_pairs(): area += u.cross(v)
print(area)
let ring: [0..5]Int = [3, 1, 4, 1, 5]
var best = 0
for (a, b) in ring.cyclic_pairs():
    if a + b > 8: break
    best max= a + b
print(best, [a * b for (a, b) in [2, 3, 5].cyclic_pairs()])
var ts = [(2, 0), (1, 1), (2, 2), (1, 3), (3, 4)]
let k: Int = read()
let flip = k == 1
if flip: ts.sort(by: t => desc(t.0)) else: ts.sort(by: t => t.0)
print(ts)
if not flip: ts.sort(by: t => desc(t.0)) else: ts.sort(by: t => t.0)
print(ts)
print(sorted([(1, 0), (2, 1), (1, 2)], by: t => desc(-t.0)))
