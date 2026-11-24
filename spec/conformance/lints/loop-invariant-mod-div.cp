// expect: L0415
type Z = Mod<998_244_353>
let d = Z(3)
var xs = [Z(1), Z(2), Z(3)]
for i in 0..xs.len:
    xs[i] = xs[i] / d // <- here
print(xs)
