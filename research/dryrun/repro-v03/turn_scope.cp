// std.geometry's `turn` is not reachable from a program without a `use`, though `Pt` is.
let (a, b, c) = (Pt(0, 0), Pt(1, 0), Pt(0, 1))
print(turn(a, b, c))
