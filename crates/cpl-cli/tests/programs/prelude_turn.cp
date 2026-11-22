// `turn` comes with `Pt` from the prelude, no `use` needed.
let (a, b, c) = (Pt(0, 0), Pt(1, 0), Pt(0, 1))
print(turn(a, b, c), turn(a, c, b), turn(a, b, Pt(2, 0)))
