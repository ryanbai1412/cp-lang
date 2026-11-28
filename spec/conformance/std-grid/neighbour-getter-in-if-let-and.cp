// expect: ok, output "2 2 3"
// rule: 04b.5.1
// The Bricks (cf1404e) shape: `if let q = p.up, black[q]`, and `p.step(Dir4.up)` with the
// constant agrees with the getter (lib/graphs-trees.md).
domain R = 0..2
domain C = 0..3
var black: [R, C]Bool = false
for (r, c) in [(0, 0), (0, 2), (1, 1)]: black[R(r), C(c)] = true
var (ups, lefts, same) = (0, 0, 0)
for r in R:
    for c in C:
        let p = (r, c)
        if let q = p.up, black[q]: ups += 1
        if let q = p.left, black[q]: lefts += 1
        if p.step(Dir4.down) == p.down and p.step(Dir4.right) == p.right and p.down != None: same += 1
print(ups, lefts, same)
