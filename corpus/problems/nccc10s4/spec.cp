// nccc10s4: n (3 or 4) segments; the answer depends only on how many share the
// most common direction.
let n = read<3..=4>()
let segs = read<[n](a: Pt, b: Pt)>()
let most = segs.tally(by: s => (s.b - s.a).unit_dir()).values.max()!
let answer = match (n, most):
    (3, 1) => 1
    (4, 1) => 2
    (4, 2) => 1
    _ => 0
print(answer)
