let n = read<3..=4>()
let segs = read<[n](a: Point, b: Point)>()
let most = segs.counts(by: s => (s.b - s.a).direction()).values.max()!
print(match (n, most):
    (3, 1) => 1
    (4, 1) => 2
    (4, 2) => 1
    _ => 0)
