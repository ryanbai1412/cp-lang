type Z = Mod<998_244_353>
let d = Z(3)
var xs = [Z(1), Z(2), Z(3)]
let inv_d = d.inv()
for i in 0..xs.len:
    xs[i] = xs[i] * inv_d
print(xs)
